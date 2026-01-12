use gtk4::gdk::Monitor;
use gtk4::prelude::*;

use crate::config::MonitorSelection;

/// Manages monitor detection and selection
pub struct MonitorManager;

impl MonitorManager {
    /// Get the target monitor based on configuration
    pub fn get_target_monitor(selection: &MonitorSelection, name: Option<&str>) -> Option<Monitor> {
        let display = gtk4::gdk::Display::default()?;
        let monitors = display.monitors();

        match selection {
            MonitorSelection::Primary => {
                // GTK4 doesn't have a direct "primary" concept, use first monitor
                monitors.item(0).and_then(|m| m.downcast::<Monitor>().ok())
            }
            MonitorSelection::Focused => {
                // For now, return the first monitor
                // In a full implementation, we'd track the focused output
                monitors.item(0).and_then(|m| m.downcast::<Monitor>().ok())
            }
            MonitorSelection::Named => {
                if let Some(target_name) = name {
                    for i in 0..monitors.n_items() {
                        if let Some(monitor) =
                            monitors.item(i).and_then(|m| m.downcast::<Monitor>().ok())
                        {
                            if let Some(connector) = monitor.connector() {
                                if connector == target_name {
                                    return Some(monitor);
                                }
                            }
                        }
                    }
                }
                None
            }
            MonitorSelection::All => {
                // For "all", we'd create windows on each monitor
                // For now, just return the first one
                monitors.item(0).and_then(|m| m.downcast::<Monitor>().ok())
            }
        }
    }

    /// List all available monitors
    pub fn list_monitors() -> Vec<String> {
        let mut result = Vec::new();

        if let Some(display) = gtk4::gdk::Display::default() {
            let monitors = display.monitors();
            for i in 0..monitors.n_items() {
                if let Some(monitor) = monitors.item(i).and_then(|m| m.downcast::<Monitor>().ok()) {
                    if let Some(connector) = monitor.connector() {
                        result.push(connector.to_string());
                    }
                }
            }
        }

        result
    }
}
