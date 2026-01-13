mod store;

pub use store::HistoryStore;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A stored notification entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: Option<String>,
    pub urgency: String,
    pub timestamp: DateTime<Utc>,
    pub actions: Vec<String>,
    pub dismissed: bool,
    pub expired: bool,
}

impl HistoryEntry {
    pub fn new(
        id: u32,
        app_name: String,
        summary: String,
        body: String,
        icon: Option<String>,
        urgency: String,
        actions: Vec<String>,
    ) -> Self {
        Self {
            id,
            app_name,
            summary,
            body,
            icon,
            urgency,
            timestamp: Utc::now(),
            actions,
            dismissed: false,
            expired: false,
        }
    }
}
