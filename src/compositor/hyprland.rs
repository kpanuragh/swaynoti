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
        info!("🔍 focus_window called with app_name: '{}'", app_name);
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
            info!("  Trying: {}", cmd);
            if let Some(response) = Self::send_command(cmd) {
                info!("    Response: '{}'", response.trim());
                // Note: Hyprland returns 'ok' even if the window isn't found
                // So we can't rely on the response. Always try fuzzy matching as fallback.
                if response.trim() == "ok" || response.is_empty() {
                    // Direct match might have worked, but we can't be sure
                    // Continue to fuzzy matching for verification
                    info!("  Command accepted, checking if focus succeeded...");
                    break; // Exit strategies loop and try fuzzy matching
                }
            }
        }

        // Fallback: search all windows for fuzzy match
        info!("  No direct match, trying fuzzy search...");
        if let Some(windows) = Self::send_command("clients") {
            info!("  Got window list, searching for match...");
            if let Some(addr) = Self::find_window_by_fuzzy_match(&windows, app_name, &app_lower) {
                info!("  ✓ Found matching window address: {}", addr);

                // First, try to get the window's workspace
                if let Some(workspace_info) = Self::find_window_workspace(&windows, &addr) {
                    info!("    Window is on workspace: {}", workspace_info);
                    // Switch to that workspace first
                    let switch_cmd = format!("dispatch workspace {}", workspace_info);
                    if let Some(response) = Self::send_command(&switch_cmd) {
                        info!("    Workspace switch response: '{}'", response.trim());
                    }
                }

                // Now focus the window
                if let Some(response) =
                    Self::send_command(&format!("dispatch focuswindow address:{}", addr))
                {
                    info!("    Focus response: '{}'", response.trim());
                    if response.trim() == "ok" || response.is_empty() {
                        info!("✓ Focused window for app via fuzzy match: {}", app_name);
                        return;
                    }
                }
            } else {
                info!("  ✗ No matching window found in fuzzy search");
            }
        }

        warn!("✗ Could not focus window for app: {}", app_name);
    }

    /// Find the workspace ID for a given window address
    fn find_window_workspace(output: &str, target_addr: &str) -> Option<String> {
        let mut current_addr: Option<String> = None;

        for line in output.lines() {
            if line.starts_with("Window ") && line.contains("->") {
                if let Some(addr) = line
                    .strip_prefix("Window ")
                    .and_then(|rest| rest.split_whitespace().next())
                {
                    current_addr = Some(addr.to_string());
                }
            }

            if let Some(addr) = &current_addr {
                if addr == target_addr {
                    // Found our window, now look for workspace line
                    if line.contains("workspace:") {
                        // Extract workspace number: "workspace: 1 (1)" -> "1"
                        if let Some(ws_part) = line.split("workspace:").nth(1) {
                            if let Some(ws_num) = ws_part.trim().split_whitespace().next() {
                                return Some(ws_num.to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Find window by fuzzy matching app name in class/title/initialclass
    fn find_window_by_fuzzy_match(output: &str, app_name: &str, app_lower: &str) -> Option<String> {
        // Parse hyprctl clients output format:
        // Window <addr> -> <title>:
        //   class: <class>
        //   initialClass: <initialclass>
        // etc.

        info!(
            "    Fuzzy search: looking for '{}' (lowercase: '{}') in {} lines",
            app_name,
            app_lower,
            output.lines().count()
        );
        let mut current_addr: Option<String> = None;
        let mut window_count = 0;

        for line in output.lines() {
            // Check for window header: "Window 5a04452b1f20 -> Alacritty:"
            if line.starts_with("Window ") && line.contains("->") {
                if let Some(addr) = line
                    .strip_prefix("Window ")
                    .and_then(|rest| rest.split_whitespace().next())
                {
                    current_addr = Some(addr.to_string());
                    window_count += 1;
                    info!("    Window #{}: {}", window_count, addr);
                }
            }

            // Check if current window matches app name
            if let Some(addr) = &current_addr {
                let line_lower = line.to_lowercase();

                // Match on class, initialClass, or title containing the app name
                if line.contains("class:")
                    || line.contains("initialClass:")
                    || line.contains("title:")
                {
                    info!("      Checking: {}", line.trim());
                    if line_lower.contains(app_lower)
                        || line.to_lowercase().contains(&app_name.to_lowercase())
                    {
                        info!("      ✓ MATCH FOUND! Returning address: {}", addr);
                        return Some(addr.clone());
                    }
                }
            }
        }

        info!(
            "    Fuzzy search completed, checked {} windows, no match found",
            window_count
        );
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
