use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use async_channel::Sender;
use chrono::Utc;
use parking_lot::RwLock;
use tokio::time::{Duration, sleep};
use tracing::{debug, info, warn};

use super::{Notification, Urgency};
use crate::config::Config;

/// Events sent to the UI thread
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Show a new notification
    Show(Notification),
    /// Update an existing notification
    Update(u32, Notification),
    /// Close a notification
    Close(u32),
    /// Reposition all notifications
    Reposition,
}

/// Events sent from UI back to the manager
#[derive(Debug, Clone)]
pub enum ActionEvent {
    /// An action was invoked
    ActionInvoked { id: u32, action_key: String },
    /// A notification was dismissed by user
    Dismissed { id: u32 },
    /// Mouse entered notification
    Hovered { id: u32 },
    /// Mouse left notification
    Unhovered { id: u32 },
}

/// Reason for closing a notification (FreeDesktop spec)
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum CloseReason {
    Expired = 1,
    Dismissed = 2,
    CloseCall = 3,
    Undefined = 4,
}

/// Manages active notifications and their lifecycle
pub struct NotificationManager {
    /// Active notifications by ID
    notifications: RwLock<HashMap<u32, Notification>>,

    /// Display order of notification IDs
    display_order: RwLock<Vec<u32>>,

    /// Next notification ID
    next_id: AtomicU32,

    /// Configuration
    config: Arc<RwLock<Config>>,

    /// Channel to send UI events
    ui_sender: Sender<UiEvent>,

    /// Channel to receive close signals (for D-Bus)
    close_sender: Sender<(u32, CloseReason)>,
}

impl NotificationManager {
    pub fn new(
        config: Arc<RwLock<Config>>,
        ui_sender: Sender<UiEvent>,
        close_sender: Sender<(u32, CloseReason)>,
    ) -> Self {
        Self {
            notifications: RwLock::new(HashMap::new()),
            display_order: RwLock::new(Vec::new()),
            next_id: AtomicU32::new(1),
            config,
            ui_sender,
            close_sender,
        }
    }

    /// Generate a new unique notification ID
    fn generate_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Add a new notification or replace an existing one
    pub async fn add_notification(&self, mut notification: Notification) -> u32 {
        let id = if notification.replaces_id > 0 {
            // Check if the notification to replace exists
            let exists = {
                let notifications = self.notifications.read();
                notifications.contains_key(&notification.replaces_id)
            };

            if exists {
                notification.replaces_id
            } else {
                self.generate_id()
            }
        } else {
            self.generate_id()
        };

        notification.id = id;

        // Calculate expiration time
        let timeout = self.calculate_timeout(&notification);
        if timeout > 0 {
            notification.expires_at = Some(Utc::now() + chrono::Duration::milliseconds(timeout as i64));
        }

        let is_replacement = {
            let mut notifications = self.notifications.write();
            let is_replacement = notifications.contains_key(&id);
            notifications.insert(id, notification.clone());
            is_replacement
        };

        if is_replacement {
            debug!("Replacing notification {}", id);
            let _ = self.ui_sender.send(UiEvent::Update(id, notification.clone())).await;
        } else {
            // Add to display order
            {
                let mut order = self.display_order.write();
                let config = self.config.read();

                match config.general.sort_order {
                    crate::config::SortOrder::NewestFirst => order.insert(0, id),
                    crate::config::SortOrder::OldestFirst => order.push(id),
                    crate::config::SortOrder::UrgencyDescending => {
                        // Insert based on urgency
                        let pos = order.iter().position(|&existing_id| {
                            if let Some(existing) = self.notifications.read().get(&existing_id) {
                                (notification.hints.urgency as u8) > (existing.hints.urgency as u8)
                            } else {
                                true
                            }
                        }).unwrap_or(order.len());
                        order.insert(pos, id);
                    }
                }
            }

            info!("Added notification {}: {}", id, notification.summary);
            let _ = self.ui_sender.send(UiEvent::Show(notification.clone())).await;
        }

        // Schedule expiration if timeout > 0
        if timeout > 0 {
            self.schedule_expiration(id, timeout).await;
        }

        id
    }

    /// Calculate the timeout for a notification
    fn calculate_timeout(&self, notification: &Notification) -> i32 {
        let config = self.config.read();

        if notification.expire_timeout == 0 {
            // Never expires
            return 0;
        }

        if notification.expire_timeout > 0 {
            // Use provided timeout
            return notification.expire_timeout;
        }

        // Use server default based on urgency
        match notification.hints.urgency {
            Urgency::Low => config.timeouts.low,
            Urgency::Normal => config.timeouts.normal,
            Urgency::Critical => config.timeouts.critical,
        }
    }

    /// Schedule a notification to expire after the given timeout
    async fn schedule_expiration(&self, id: u32, timeout_ms: i32) {
        let ui_sender = self.ui_sender.clone();
        let close_sender = self.close_sender.clone();
        let notifications = Arc::new(&self.notifications);

        tokio::spawn(async move {
            sleep(Duration::from_millis(timeout_ms as u64)).await;

            // Check if notification is still active and not hovered
            // Note: In real implementation, we'd need proper Arc handling here
            debug!("Notification {} expired", id);
            let _ = ui_sender.send(UiEvent::Close(id)).await;
            let _ = close_sender.send((id, CloseReason::Expired)).await;
        });
    }

    /// Close a notification
    pub async fn close_notification(&self, id: u32, reason: CloseReason) {
        let existed = {
            let mut notifications = self.notifications.write();
            notifications.remove(&id).is_some()
        };

        if existed {
            {
                let mut order = self.display_order.write();
                order.retain(|&x| x != id);
            }

            info!("Closed notification {} (reason: {:?})", id, reason);
            let _ = self.ui_sender.send(UiEvent::Close(id)).await;
            let _ = self.close_sender.send((id, reason)).await;
        }
    }

    /// Get a notification by ID
    pub fn get_notification(&self, id: u32) -> Option<Notification> {
        self.notifications.read().get(&id).cloned()
    }

    /// Get all visible notifications in display order
    pub fn get_visible_notifications(&self) -> Vec<Notification> {
        let config = self.config.read();
        let order = self.display_order.read();
        let notifications = self.notifications.read();

        order
            .iter()
            .take(config.general.max_visible as usize)
            .filter_map(|id| notifications.get(id).cloned())
            .collect()
    }

    /// Get the count of active notifications
    pub fn count(&self) -> usize {
        self.notifications.read().len()
    }

    /// Handle action invoked event
    pub async fn invoke_action(&self, id: u32, action_key: &str) {
        if let Some(notification) = self.get_notification(id) {
            // Check if action exists
            if notification.actions.iter().any(|(key, _)| key == action_key) {
                info!("Action '{}' invoked on notification {}", action_key, id);
                // The D-Bus server will emit the ActionInvoked signal
            }
        }
    }

    /// Set hover state for a notification
    pub fn set_hovered(&self, id: u32, hovered: bool) {
        let mut notifications = self.notifications.write();
        if let Some(notification) = notifications.get_mut(&id) {
            notification.is_hovered = hovered;
            debug!("Notification {} hover state: {}", id, hovered);
        }
    }
}
