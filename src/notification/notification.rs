use super::Urgency;
use chrono::{DateTime, Utc};

/// Core notification structure
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique notification ID assigned by the daemon
    pub id: u32,

    /// Application name that sent the notification
    pub app_name: String,

    /// Notification to replace (0 = new notification)
    pub replaces_id: u32,

    /// Icon specification (icon name, file path, or empty)
    pub app_icon: String,

    /// Summary/title of the notification
    pub summary: String,

    /// Body text (may contain Pango markup)
    pub body: String,

    /// Action buttons: Vec<(action_key, display_label)>
    pub actions: Vec<(String, String)>,

    /// Hints from the notification
    pub hints: NotificationHints,

    /// Expiration timeout in milliseconds (-1 = server default, 0 = never)
    pub expire_timeout: i32,

    /// Calculated expiration time
    pub expires_at: Option<DateTime<Utc>>,

    /// When the notification was created
    pub created_at: DateTime<Utc>,

    /// Whether the notification is currently being hovered
    pub is_hovered: bool,
}

impl Notification {
    /// Create a new notification with the given parameters
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u32,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: NotificationHints,
        expire_timeout: i32,
    ) -> Self {
        // Parse actions into (key, label) pairs
        let actions = actions
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some((chunk[0].clone(), chunk[1].clone()))
                } else {
                    None
                }
            })
            .collect();

        Self {
            id,
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
            expires_at: None,
            created_at: Utc::now(),
            is_hovered: false,
        }
    }

    /// Check if this notification has a default action
    pub fn has_default_action(&self) -> bool {
        self.actions.iter().any(|(key, _)| key == "default")
    }

    /// Get the progress value if this is a progress notification
    pub fn progress(&self) -> Option<u8> {
        self.hints.value.map(|v| v.clamp(0, 100) as u8)
    }

    /// Check if this notification should be persisted in history
    pub fn should_persist(&self) -> bool {
        !self.hints.transient
    }

    /// Check if this notification is resident (stays until dismissed)
    pub fn is_resident(&self) -> bool {
        self.hints.resident
    }
}

/// Notification hints parsed from D-Bus
#[derive(Debug, Clone, Default)]
pub struct NotificationHints {
    /// Urgency level (0=low, 1=normal, 2=critical)
    pub urgency: Urgency,

    /// Category string (e.g., "email", "transfer")
    pub category: Option<String>,

    /// Desktop entry name
    pub desktop_entry: Option<String>,

    /// Image data (raw pixels)
    pub image_data: Option<ImageData>,

    /// Image path
    pub image_path: Option<String>,

    /// Sound file to play
    pub sound_file: Option<String>,

    /// Sound name (from sound theme)
    pub sound_name: Option<String>,

    /// Suppress sound
    pub suppress_sound: bool,

    /// Transient (should not be persisted)
    pub transient: bool,

    /// X position hint
    pub x: Option<i32>,

    /// Y position hint
    pub y: Option<i32>,

    /// Action icons (actions are icon names)
    pub action_icons: bool,

    /// Progress value (0-100)
    pub value: Option<i32>,

    /// Resident notification (stays until explicitly dismissed)
    pub resident: bool,

    /// Inline reply support (for messaging apps)
    pub inline_reply: bool,
}

/// Raw image data from D-Bus
#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: i32,
    pub height: i32,
    pub rowstride: i32,
    pub has_alpha: bool,
    pub bits_per_sample: i32,
    pub channels: i32,
    pub data: Vec<u8>,
}
