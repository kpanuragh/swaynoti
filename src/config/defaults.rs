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

/* Inline reply */
.notification .inline-reply {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid rgba(69, 71, 90, 0.3);
}

.notification .reply-entry {
    background: rgba(30, 30, 46, 0.8);
    border: 1px solid rgba(69, 71, 90, 0.5);
    border-radius: 6px;
    padding: 6px 10px;
    color: #cdd6f4;
    font-size: 13px;
}

.notification .reply-send {
    background: rgba(137, 180, 250, 0.2);
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    color: #89b4fa;
    font-size: 12px;
}

.notification .reply-send:hover {
    background: rgba(137, 180, 250, 0.3);
}

/* Notification Center */
.notification-center {
    background-color: rgba(30, 30, 46, 0.98);
    border-radius: 12px;
    border: 1px solid rgba(69, 71, 90, 0.5);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}

.notification-center-container {
    background-color: transparent;
}

.notification-center-header {
    padding: 16px;
    border-bottom: 1px solid rgba(69, 71, 90, 0.3);
}

.notification-center-title {
    font-size: 18px;
    font-weight: 600;
    color: #cdd6f4;
}

.notification-center-footer {
    padding: 12px 16px;
    border-top: 1px solid rgba(69, 71, 90, 0.3);
}

.clear-all-button {
    background: rgba(243, 139, 168, 0.15);
    border: none;
    border-radius: 6px;
    padding: 8px 16px;
    font-size: 13px;
    color: #f38ba8;
}

.clear-all-button:hover {
    background: rgba(243, 139, 168, 0.25);
}

.notification-list {
    background: transparent;
    padding: 8px;
}

.notification-list row {
    background: transparent;
    padding: 4px 0;
    min-height: 40px;
}

.notification-list row:hover {
    background: rgba(69, 71, 90, 0.1);
}

.app-group-header {
    background: rgba(69, 71, 90, 0.3);
    border-radius: 8px;
    padding: 8px 12px;
    margin: 4px 0;
    min-height: 36px;
}

.app-group-header .app-icon {
    opacity: 0.9;
    min-width: 24px;
    min-height: 24px;
}

.app-group-header .app-name {
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
}

.notification-count {
    background: rgba(137, 180, 250, 0.25);
    color: #89b4fa;
    border-radius: 12px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    min-width: 24px;
}

.notification-entry {
    margin: 2px 0 2px 24px;
    padding: 8px 12px;
    background: rgba(49, 50, 68, 0.4);
    border-radius: 6px;
    min-height: 48px;
}

.notification-entry:hover {
    background: rgba(49, 50, 68, 0.6);
}

.entry-summary {
    font-size: 13px;
    font-weight: 500;
    color: #cdd6f4;
}

.entry-body {
    font-size: 12px;
    color: rgba(205, 214, 244, 0.7);
}

.entry-time {
    font-size: 10px;
    color: rgba(205, 214, 244, 0.4);
    margin-top: 4px;
}

.empty-message {
    font-size: 14px;
    color: rgba(205, 214, 244, 0.5);
    font-style: italic;
}

.more-notifications {
    font-size: 11px;
    color: rgba(205, 214, 244, 0.5);
    font-style: italic;
}

/* Media Player Widget */
.media-widget {
    background: rgba(49, 50, 68, 0.6);
    border-radius: 10px;
    padding: 12px;
    margin: 8px;
    border-bottom: 1px solid rgba(69, 71, 90, 0.3);
}

.media-widget .album-art {
    border-radius: 8px;
    background: rgba(30, 30, 46, 0.5);
    min-width: 64px;
    min-height: 64px;
}

.media-widget .media-title {
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
}

.media-widget .media-artist {
    font-size: 12px;
    color: rgba(205, 214, 244, 0.7);
}

.media-widget .media-controls {
    margin-left: 8px;
}

.media-widget .media-button {
    background: transparent;
    border: none;
    border-radius: 50%;
    min-width: 36px;
    min-height: 36px;
    padding: 6px;
    color: rgba(205, 214, 244, 0.8);
}

.media-widget .media-button:hover {
    background: rgba(137, 180, 250, 0.2);
    color: #89b4fa;
}

.media-widget .media-button.play-pause {
    min-width: 42px;
    min-height: 42px;
    background: rgba(137, 180, 250, 0.15);
}

.media-widget .media-button.play-pause:hover {
    background: rgba(137, 180, 250, 0.3);
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
