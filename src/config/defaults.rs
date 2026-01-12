/// Default CSS theme embedded in the binary
pub const DEFAULT_CSS: &str = r#"
/* Notification container */
.notification {
    background-color: rgba(30, 30, 46, 0.95);
    border-radius: 12px;
    padding: 12px;
    margin: 4px;
    border: 1px solid rgba(69, 71, 90, 0.5);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.notification:hover {
    background-color: rgba(49, 50, 68, 0.98);
}

/* Urgency levels */
.notification.low {
    border-left: 4px solid #a6e3a1;
}

.notification.normal {
    border-left: 4px solid #89b4fa;
}

.notification.critical {
    border-left: 4px solid #f38ba8;
    background-color: rgba(243, 139, 168, 0.15);
}

/* App name */
.notification .app-name {
    font-size: 11px;
    font-weight: 500;
    color: rgba(205, 214, 244, 0.6);
    margin-bottom: 4px;
}

/* Summary/title */
.notification .summary {
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
    margin-bottom: 4px;
}

/* Body text */
.notification .body {
    font-size: 13px;
    color: rgba(205, 214, 244, 0.85);
}

/* Icon */
.notification .icon {
    min-width: 48px;
    min-height: 48px;
    margin-right: 12px;
    border-radius: 8px;
}

/* Action buttons container */
.notification .actions {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid rgba(69, 71, 90, 0.3);
}

/* Action button */
.notification .action-button {
    background: rgba(137, 180, 250, 0.15);
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    margin-right: 6px;
    font-size: 12px;
    color: #89b4fa;
}

.notification .action-button:hover {
    background: rgba(137, 180, 250, 0.25);
}

/* Progress bar */
.notification .progress trough {
    background: rgba(205, 214, 244, 0.1);
    border-radius: 4px;
    min-height: 6px;
}

.notification .progress progress {
    background: #89b4fa;
    border-radius: 4px;
}

/* Close button */
.notification .close-button {
    background: transparent;
    border: none;
    border-radius: 50%;
    min-width: 24px;
    min-height: 24px;
    padding: 4px;
    opacity: 0.5;
}

.notification .close-button:hover {
    opacity: 1;
    background: rgba(243, 139, 168, 0.2);
}

/* Timestamp */
.notification .timestamp {
    font-size: 10px;
    color: rgba(205, 214, 244, 0.4);
}

/* Window background (transparent for layer-shell) */
window {
    background-color: transparent;
}
"#;

/// Default TOML configuration
pub const DEFAULT_CONFIG: &str = r#"
[general]
max_visible = 5
sort_order = "newest-first"
markup = true

[appearance]
width = 350
max_height = 200
border_radius = 12
gap = 8
opacity = 0.95
icon_size = 48
show_app_name = true

[appearance.animations]
enabled = true
duration_ms = 200
slide_direction = "right"

[positioning]
anchor = "top-right"
layer = "overlay"
stacking = "vertical"

[positioning.margin]
top = 10
right = 10
bottom = 10
left = 10

[positioning.monitor]
selection = "focused"

[timeouts]
default = 5000
low = 3000
normal = 5000
critical = 0

[history]
enabled = true
max_entries = 100

[sound]
enabled = false
"#;
