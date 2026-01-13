use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tracing::{debug, info};

use super::HistoryEntry;
use crate::config::ConfigLoader;

/// SQLite-backed notification history store
pub struct HistoryStore {
    conn: Mutex<Connection>,
    max_entries: u32,
}

impl HistoryStore {
    /// Create a new history store
    pub fn new(max_entries: u32) -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open history database: {:?}", db_path))?;

        let store = Self {
            conn: Mutex::new(conn),
            max_entries,
        };

        store.init_schema()?;
        info!("History store initialized at {:?}", db_path);

        Ok(store)
    }

    /// Get database path
    fn get_db_path() -> Result<PathBuf> {
        let data_dir = ConfigLoader::data_dir().context("Could not determine data directory")?;
        Ok(data_dir.join("history.db"))
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY,
                notification_id INTEGER NOT NULL,
                app_name TEXT NOT NULL,
                summary TEXT NOT NULL,
                body TEXT DEFAULT '',
                icon TEXT,
                urgency TEXT DEFAULT 'normal',
                timestamp TEXT NOT NULL,
                actions TEXT DEFAULT '[]',
                dismissed INTEGER DEFAULT 0,
                expired INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON notifications(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_name ON notifications(app_name)",
            [],
        )?;

        Ok(())
    }

    /// Add a notification to history
    pub fn add(&self, entry: &HistoryEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO notifications (notification_id, app_name, summary, body, icon, urgency, timestamp, actions, dismissed, expired)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entry.id,
                entry.app_name,
                entry.summary,
                entry.body,
                entry.icon,
                entry.urgency,
                entry.timestamp.to_rfc3339(),
                serde_json::to_string(&entry.actions).unwrap_or_default(),
                entry.dismissed as i32,
                entry.expired as i32,
            ],
        )?;

        debug!("Added notification {} to history", entry.id);

        // Cleanup old entries
        drop(conn);
        self.cleanup()?;

        Ok(())
    }

    /// Mark a notification as dismissed
    pub fn mark_dismissed(&self, notification_id: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notifications SET dismissed = 1 WHERE notification_id = ?1",
            params![notification_id],
        )?;
        Ok(())
    }

    /// Mark a notification as expired
    pub fn mark_expired(&self, notification_id: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notifications SET expired = 1 WHERE notification_id = ?1",
            params![notification_id],
        )?;
        Ok(())
    }

    /// Get all history entries
    pub fn get_all(&self) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT notification_id, app_name, summary, body, icon, urgency, timestamp, actions, dismissed, expired
             FROM notifications ORDER BY timestamp DESC"
        )?;

        let entries = stmt.query_map([], |row| {
            let timestamp_str: String = row.get(6)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let actions_str: String = row.get(7)?;
            let actions: Vec<String> = serde_json::from_str(&actions_str).unwrap_or_default();

            Ok(HistoryEntry {
                id: row.get(0)?,
                app_name: row.get(1)?,
                summary: row.get(2)?,
                body: row.get(3)?,
                icon: row.get(4)?,
                urgency: row.get(5)?,
                timestamp,
                actions,
                dismissed: row.get::<_, i32>(8)? != 0,
                expired: row.get::<_, i32>(9)? != 0,
            })
        })?;

        Ok(entries.filter_map(|e| e.ok()).collect())
    }

    /// Get history entries for a specific app
    pub fn get_by_app(&self, app_name: &str) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT notification_id, app_name, summary, body, icon, urgency, timestamp, actions, dismissed, expired
             FROM notifications WHERE app_name = ?1 ORDER BY timestamp DESC"
        )?;

        let entries = stmt.query_map([app_name], |row| {
            let timestamp_str: String = row.get(6)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let actions_str: String = row.get(7)?;
            let actions: Vec<String> = serde_json::from_str(&actions_str).unwrap_or_default();

            Ok(HistoryEntry {
                id: row.get(0)?,
                app_name: row.get(1)?,
                summary: row.get(2)?,
                body: row.get(3)?,
                icon: row.get(4)?,
                urgency: row.get(5)?,
                timestamp,
                actions,
                dismissed: row.get::<_, i32>(8)? != 0,
                expired: row.get::<_, i32>(9)? != 0,
            })
        })?;

        Ok(entries.filter_map(|e| e.ok()).collect())
    }

    /// Get recent notifications (grouped by app)
    pub fn get_grouped(&self, limit: usize) -> Result<Vec<(String, Vec<HistoryEntry>)>> {
        let all = self.get_all()?;
        let mut groups: std::collections::HashMap<String, Vec<HistoryEntry>> =
            std::collections::HashMap::new();

        for entry in all.into_iter().take(limit) {
            groups
                .entry(entry.app_name.clone())
                .or_default()
                .push(entry);
        }

        let mut result: Vec<_> = groups.into_iter().collect();
        result.sort_by(|a, b| {
            let a_time = a.1.first().map(|e| e.timestamp).unwrap_or_else(Utc::now);
            let b_time = b.1.first().map(|e| e.timestamp).unwrap_or_else(Utc::now);
            b_time.cmp(&a_time)
        });

        Ok(result)
    }

    /// Clear all history
    pub fn clear(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notifications", [])?;
        info!("History cleared");
        Ok(())
    }

    /// Delete a specific entry
    pub fn delete(&self, notification_id: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM notifications WHERE notification_id = ?1",
            params![notification_id],
        )?;
        Ok(())
    }

    /// Get notification count
    pub fn count(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let count: u32 =
            conn.query_row("SELECT COUNT(*) FROM notifications", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Cleanup old entries to maintain max_entries limit
    fn cleanup(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM notifications WHERE id NOT IN (
                SELECT id FROM notifications ORDER BY timestamp DESC LIMIT ?1
            )",
            params![self.max_entries],
        )?;
        Ok(())
    }
}
