/// Capabilities advertised by the notification server
pub const CAPABILITIES: &[&str] = &[
    "actions",
    "action-icons",
    "body",
    "body-hyperlinks",
    "body-markup",
    "icon-static",
    "persistence",
];

/// Reasons for closing a notification (FreeDesktop spec)
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum CloseReason {
    /// The notification expired
    Expired = 1,
    /// The notification was dismissed by the user
    Dismissed = 2,
    /// The notification was closed by a call to CloseNotification
    CloseCall = 3,
    /// Undefined/reserved reason
    Undefined = 4,
}

impl From<CloseReason> for u32 {
    fn from(reason: CloseReason) -> Self {
        reason as u32
    }
}

/// Server information
pub struct ServerInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub version: &'static str,
    pub spec_version: &'static str,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: "swaynoti",
            vendor: "swaynoti",
            version: env!("CARGO_PKG_VERSION"),
            spec_version: "1.2",
        }
    }
}
