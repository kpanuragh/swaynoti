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
        let app_lower = app_name.to_lowercase();

        // Try different matching strategies
        let strategies = [
            format!("dispatch focuswindow class:{}", app_name),
            format!("dispatch focuswindow class:{}", app_lower),
            format!("dispatch focuswindow title:{}", app_name),
            format!("dispatch focuswindow class:^{}$", app_lower),
            format!("dispatch focuswindow initialclass:{}", app_lower),
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

        // Fallback: search all windows for fuzzy match
        debug!("No direct match found, searching all windows for fuzzy match");
        if let Some(windows) = Self::send_command("j/clients") {
            if let Some(addr) = Self::find_window_by_fuzzy_match(&windows, app_name, &app_lower) {
                debug!("Found matching window address: {}", addr);
                if let Some(response) = Self::send_command(&format!("dispatch focuswindow address:{}", addr)) {
                    if response.trim() == "ok" || response.is_empty() {
                        info!("Focused window for app via fuzzy match: {}", app_name);
                        return;
                    }
                }
            }
        }

        warn!("Could not focus window for app: {}", app_name);
    }

    /// Find window by fuzzy matching app name in class/title/initialclass
    fn find_window_by_fuzzy_match(output: &str, app_name: &str, app_lower: &str) -> Option<String> {
        // Parse hyprctl clients output format:
        // Window <addr> -> <title>:
        //   class: <class>
        //   initialClass: <initialclass>
        // etc.

        let mut current_addr: Option<String> = None;

        for line in output.lines() {
            // Check for window header: "Window 5a04452b1f20 -> Alacritty:"
            if line.starts_with("Window ") && line.contains("->") {
                if let Some(addr) = line.strip_prefix("Window ").and_then(|rest| rest.split_whitespace().next()) {
                    current_addr = Some(addr.to_string());
                }
            }

            // Check if current window matches app name
            if let Some(addr) = &current_addr {
                let line_lower = line.to_lowercase();

                // Match on class, initialClass, or title containing the app name
                if line.contains("class:") || line.contains("initialClass:") || line.contains("title:") {
                    if line_lower.contains(app_lower) || line.to_lowercase().contains(&app_name.to_lowercase()) {
                        debug!("Found matching window: class/title contains '{}'", app_name);
                        return Some(addr.clone());
                    }
                }
            }
        }

        None
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
