mod hyprland;

pub use hyprland::HyprlandIpc;

use std::process::Command;
use tracing::{debug, info, warn};

/// Compositor abstraction for window management
pub struct CompositorIpc;

impl CompositorIpc {
    /// Focus a window by app name/class
    pub fn focus_window(app_name: &str) {
        info!("CompositorIpc::focus_window called with: '{}'", app_name);
        // Try Hyprland first
        if HyprlandIpc::is_available() {
            info!("✓ Hyprland detected, delegating to HyprlandIpc");
            HyprlandIpc::focus_window(app_name);
            return;
        }

        // Fallback: try swaymsg for Sway
        if Self::is_sway() {
            info!("✓ Sway detected, delegating to swaymsg");
            Self::sway_focus_window(app_name);
            return;
        }

        warn!("✗ No supported compositor found for window focusing");
    }

    /// Check if running on Sway
    fn is_sway() -> bool {
        std::env::var("SWAYSOCK").is_ok()
    }

    /// Focus window using swaymsg
    fn sway_focus_window(app_name: &str) {
        // Try app_id first
        let result = Command::new("swaymsg")
            .arg(&format!("[app_id={}] focus", app_name))
            .output();

        match result {
            Ok(output) => {
                if !output.status.success() {
                    // Try with class instead of app_id
                    debug!("app_id focus failed, trying class");
                    let _ = Command::new("swaymsg")
                        .arg(&format!("[class={}] focus", app_name))
                        .output();
                }
            }
            Err(e) => warn!("Failed to focus window via swaymsg: {}", e),
        }
    }
}
