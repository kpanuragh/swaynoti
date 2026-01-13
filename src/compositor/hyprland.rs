use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

/// Hyprland IPC client for window management
pub struct HyprlandIpc;

impl HyprlandIpc {
    /// Check if Hyprland is available
    pub fn is_available() -> bool {
        Self::get_socket_path().is_some()
    }

    /// Get Hyprland socket path
    fn get_socket_path() -> Option<PathBuf> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
        let xdg_runtime = std::env::var("XDG_RUNTIME_DIR").ok()?;
        Some(PathBuf::from(format!(
            "{}/hypr/{}/.socket.sock",
            xdg_runtime, his
        )))
    }

    /// Send a command to Hyprland
    fn send_command(command: &str) -> Option<String> {
        let socket_path = Self::get_socket_path()?;

        let mut stream = match UnixStream::connect(&socket_path) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to Hyprland socket: {}", e);
                return None;
            }
        };

        if let Err(e) = stream.write_all(command.as_bytes()) {
            error!("Failed to send command to Hyprland: {}", e);
            return None;
        }

        let mut response = String::new();
        if let Err(e) = stream.read_to_string(&mut response) {
            warn!("Failed to read Hyprland response: {}", e);
        }

        Some(response)
    }

    /// Focus a window by app name/class
    pub fn focus_window(app_name: &str) {
        // Try different matching strategies
        let strategies = [
            format!("dispatch focuswindow class:{}", app_name),
            format!("dispatch focuswindow title:{}", app_name),
            format!("dispatch focuswindow class:^{}$", app_name.to_lowercase()),
            format!("dispatch focuswindow initialclass:{}", app_name),
        ];

        for cmd in &strategies {
            debug!("Trying Hyprland command: {}", cmd);
            if let Some(response) = Self::send_command(cmd) {
                if response.trim() == "ok" || response.is_empty() {
                    info!("Focused window for app: {}", app_name);
                    return;
                }
            }
        }

        warn!("Could not focus window for app: {}", app_name);
    }

    /// Get list of windows
    pub fn get_windows() -> Option<String> {
        Self::send_command("j/clients")
    }

    /// Get active window info
    pub fn get_active_window() -> Option<String> {
        Self::send_command("j/activewindow")
    }

    /// Dispatch a Hyprland command
    pub fn dispatch(args: &str) -> Option<String> {
        Self::send_command(&format!("dispatch {}", args))
    }
}
