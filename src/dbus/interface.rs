use std::collections::HashMap;
use std::sync::Arc;

use async_channel::Receiver;
use tracing::{debug, info};
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Value};

use super::types::{ServerInfo, CAPABILITIES};
use crate::history::{HistoryEntry, HistoryStore};
use crate::notification::{
    CloseReason, ImageData, Notification, NotificationHints, NotificationManager, Urgency,
};

/// D-Bus notification server implementing org.freedesktop.Notifications
pub struct NotificationServer {
    manager: Arc<NotificationManager>,
    #[allow(dead_code)]
    close_receiver: Receiver<(u32, CloseReason)>,
    history_store: Option<Arc<HistoryStore>>,
}

impl NotificationServer {
    pub fn new(
        manager: Arc<NotificationManager>,
        close_receiver: Receiver<(u32, CloseReason)>,
    ) -> Self {
        Self {
            manager,
            close_receiver,
            history_store: None,
        }
    }

    pub fn with_history(mut self, store: Arc<HistoryStore>) -> Self {
        self.history_store = Some(store);
        self
    }

    /// Parse hints from D-Bus variant dictionary
    fn parse_hints(hints: HashMap<String, OwnedValue>) -> NotificationHints {
        let mut result = NotificationHints::default();

        for (key, value) in hints {
            match key.as_str() {
                "urgency" => {
                    if let Ok(u) = TryInto::<u8>::try_into(&*value) {
                        result.urgency = Urgency::from(u);
                    }
                }
                "category" => {
                    if let Ok(s) = TryInto::<String>::try_into(&*value) {
                        result.category = Some(s);
                    }
                }
                "desktop-entry" => {
                    if let Ok(s) = TryInto::<String>::try_into(&*value) {
                        result.desktop_entry = Some(s);
                    }
                }
                "image-data" | "image_data" | "icon_data" => {
                    if let Some(data) = Self::parse_image_data(&value) {
                        result.image_data = Some(data);
                    }
                }
                "image-path" | "image_path" => {
                    if let Ok(s) = TryInto::<String>::try_into(&*value) {
                        result.image_path = Some(s);
                    }
                }
                "sound-file" => {
                    if let Ok(s) = TryInto::<String>::try_into(&*value) {
                        result.sound_file = Some(s);
                    }
                }
                "sound-name" => {
                    if let Ok(s) = TryInto::<String>::try_into(&*value) {
                        result.sound_name = Some(s);
                    }
                }
                "suppress-sound" => {
                    if let Ok(b) = TryInto::<bool>::try_into(&*value) {
                        result.suppress_sound = b;
                    }
                }
                "transient" => {
                    if let Ok(b) = TryInto::<bool>::try_into(&*value) {
                        result.transient = b;
                    }
                }
                "x" => {
                    if let Ok(x) = TryInto::<i32>::try_into(&*value) {
                        result.x = Some(x);
                    }
                }
                "y" => {
                    if let Ok(y) = TryInto::<i32>::try_into(&*value) {
                        result.y = Some(y);
                    }
                }
                "action-icons" => {
                    if let Ok(b) = TryInto::<bool>::try_into(&*value) {
                        result.action_icons = b;
                    }
                }
                "value" => {
                    if let Ok(v) = TryInto::<i32>::try_into(&*value) {
                        result.value = Some(v);
                    }
                }
                "resident" => {
                    if let Ok(b) = TryInto::<bool>::try_into(&*value) {
                        result.resident = b;
                    }
                }
                _ => {
                    debug!("Unknown hint: {}", key);
                }
            }
        }

        result
    }

    /// Parse image data from D-Bus variant
    fn parse_image_data(value: &Value) -> Option<ImageData> {
        // Image data is a structure: (iiibiiay)
        // width, height, rowstride, has_alpha, bits_per_sample, channels, data
        if let Value::Structure(s) = value {
            let fields = s.fields();
            if fields.len() >= 7 {
                let width = TryInto::<i32>::try_into(&fields[0]).ok()?;
                let height = TryInto::<i32>::try_into(&fields[1]).ok()?;
                let rowstride = TryInto::<i32>::try_into(&fields[2]).ok()?;
                let has_alpha = TryInto::<bool>::try_into(&fields[3]).ok()?;
                let bits_per_sample = TryInto::<i32>::try_into(&fields[4]).ok()?;
                let channels = TryInto::<i32>::try_into(&fields[5]).ok()?;

                // Get the byte array
                if let Value::Array(arr) = &fields[6] {
                    let data: Vec<u8> = arr
                        .iter()
                        .filter_map(|v| TryInto::<u8>::try_into(v).ok())
                        .collect();

                    return Some(ImageData {
                        width,
                        height,
                        rowstride,
                        has_alpha,
                        bits_per_sample,
                        channels,
                        data,
                    });
                }
            }
        }
        None
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    /// Returns the capabilities of the notification server
    fn get_capabilities(&self) -> Vec<String> {
        CAPABILITIES.iter().map(|s| s.to_string()).collect()
    }

    /// Sends a notification to the notification server
    #[allow(clippy::too_many_arguments)]
    async fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        info!(
            "Received notification: app={}, summary={}, replaces={}",
            app_name, summary, replaces_id
        );

        let parsed_hints = Self::parse_hints(hints);

        let notification = Notification::new(
            0, // Will be assigned by manager
            app_name.to_string(),
            replaces_id,
            app_icon.to_string(),
            summary.to_string(),
            body.to_string(),
            actions.clone(),
            parsed_hints.clone(),
            expire_timeout,
        );

        let id = self.manager.add_notification(notification).await;

        // Save to history if not transient
        if !parsed_hints.transient {
            if let Some(ref store) = self.history_store {
                let entry = HistoryEntry {
                    id,
                    app_name: app_name.to_string(),
                    summary: summary.to_string(),
                    body: body.to_string(),
                    icon: if app_icon.is_empty() {
                        None
                    } else {
                        Some(app_icon.to_string())
                    },
                    urgency: parsed_hints.urgency.to_string(),
                    timestamp: chrono::Utc::now(),
                    actions: actions
                        .chunks(2)
                        .filter_map(|c| c.first().cloned())
                        .collect(),
                    dismissed: false,
                    expired: false,
                };
                if let Err(e) = store.add(&entry) {
                    debug!("Failed to save notification to history: {}", e);
                }
            }
        }

        id
    }

    /// Closes the notification with the given ID
    async fn close_notification(&self, id: u32) {
        info!("CloseNotification called for id={}", id);
        self.manager
            .close_notification(id, crate::notification::CloseReason::CloseCall)
            .await;
    }

    /// Returns the server information
    fn get_server_information(&self) -> (String, String, String, String) {
        let info = ServerInfo::default();
        (
            info.name.to_string(),
            info.vendor.to_string(),
            info.version.to_string(),
            info.spec_version.to_string(),
        )
    }

    /// Signal emitted when a notification is closed
    #[zbus(signal)]
    async fn notification_closed(
        emitter: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    /// Signal emitted when an action is invoked
    #[zbus(signal)]
    async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: String,
    ) -> zbus::Result<()>;
}
