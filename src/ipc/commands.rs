use serde::{Deserialize, Serialize};

/// IPC commands that can be sent to the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum IpcCommand {
    /// Dismiss a specific notification
    Dismiss { id: u32 },

    /// Dismiss all notifications
    DismissAll,

    /// Toggle Do Not Disturb mode
    ToggleDnd,

    /// Enable Do Not Disturb
    EnableDnd,

    /// Disable Do Not Disturb
    DisableDnd,

    /// Get DND status
    GetDndStatus,

    /// Show notification history panel
    ShowHistory,

    /// Hide notification history panel
    HideHistory,

    /// Get count of active notifications
    GetCount,

    /// Reload configuration
    ReloadConfig,

    /// Get list of active notifications (for waybar, etc.)
    GetNotifications,

    /// Invoke an action on a notification
    InvokeAction { id: u32, action: String },
}

/// Response from IPC commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }

    pub fn with_data(data: impl Serialize) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).ok(),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}
