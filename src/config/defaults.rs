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
    background: linear-gradient(180deg, rgba(24, 24, 37, 0.95) 0%, rgba(30, 30, 46, 0.98) 100%);
    border-radius: 16px 0 0 16px;
    border-left: 1px solid rgba(137, 180, 250, 0.3);
    border-top: 1px solid rgba(137, 180, 250, 0.2);
    border-bottom: 1px solid rgba(69, 71, 90, 0.3);
    box-shadow: -8px 0 32px rgba(0, 0, 0, 0.6);
}

.notification-center-container {
    background-color: transparent;
}

.notification-center-header {
    padding: 20px 16px;
    border-bottom: 1px solid rgba(137, 180, 250, 0.15);
    background: linear-gradient(90deg, rgba(137, 180, 250, 0.08) 0%, transparent 100%);
}

.notification-center-title {
    font-size: 20px;
    font-weight: 700;
    color: #cdd6f4;
    letter-spacing: 0.5px;
}

.notification-center-footer {
    padding: 16px;
    border-top: 1px solid rgba(69, 71, 90, 0.3);
    background: rgba(17, 17, 27, 0.5);
}

.clear-all-button {
    background: linear-gradient(135deg, rgba(243, 139, 168, 0.2) 0%, rgba(243, 139, 168, 0.1) 100%);
    border: 1px solid rgba(243, 139, 168, 0.3);
    border-radius: 8px;
    padding: 10px 20px;
    font-size: 13px;
    font-weight: 600;
    color: #f38ba8;
    transition: all 0.2s ease;
}

.clear-all-button:hover {
    background: linear-gradient(135deg, rgba(243, 139, 168, 0.3) 0%, rgba(243, 139, 168, 0.2) 100%);
    border-color: rgba(243, 139, 168, 0.5);
}

.notification-list {
    background: transparent;
    padding: 12px 8px;
}

.notification-list row {
    background: transparent;
    padding: 2px 0;
    margin: 2px 0;
    border-radius: 8px;
}

.notification-list row:hover {
    background: rgba(137, 180, 250, 0.05);
}

.app-group-header {
    background: linear-gradient(135deg, rgba(69, 71, 90, 0.4) 0%, rgba(49, 50, 68, 0.3) 100%);
    border-radius: 12px;
    border: 1px solid rgba(69, 71, 90, 0.3);
    padding: 12px 16px;
    margin: 8px 4px;
    min-height: 44px;
}

.app-group-header .app-icon {
    opacity: 1;
    min-width: 28px;
    min-height: 28px;
    margin-right: 4px;
}

.app-group-header .app-name {
    font-size: 15px;
    font-weight: 600;
    color: #cdd6f4;
    letter-spacing: 0.3px;
}

.notification-count {
    background: linear-gradient(135deg, rgba(137, 180, 250, 0.3) 0%, rgba(137, 180, 250, 0.15) 100%);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.3);
    border-radius: 16px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 700;
    min-width: 28px;
}

.notification-entry {
    margin: 4px 8px 4px 20px;
    padding: 12px 16px;
    background: linear-gradient(135deg, rgba(49, 50, 68, 0.5) 0%, rgba(39, 40, 58, 0.4) 100%);
    border-radius: 10px;
    border-left: 3px solid rgba(137, 180, 250, 0.4);
    min-height: 52px;
    transition: all 0.2s ease;
}

.notification-entry:hover {
    background: linear-gradient(135deg, rgba(59, 60, 78, 0.6) 0%, rgba(49, 50, 68, 0.5) 100%);
    border-left-color: rgba(137, 180, 250, 0.7);
}

.entry-summary {
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
    margin-bottom: 2px;
}

.entry-body {
    font-size: 12px;
    color: rgba(205, 214, 244, 0.75);
    line-height: 1.4;
}

.entry-time {
    font-size: 10px;
    color: rgba(137, 180, 250, 0.6);
    margin-top: 6px;
    font-weight: 500;
}

.empty-message {
    font-size: 15px;
    color: rgba(205, 214, 244, 0.4);
    font-style: italic;
    padding: 40px 20px;
}

.more-notifications {
    font-size: 12px;
    color: rgba(137, 180, 250, 0.5);
    font-style: italic;
    padding: 8px 20px;
}

/* Media Player Widget */
.media-widget {
    background: linear-gradient(135deg, rgba(49, 50, 68, 0.7) 0%, rgba(39, 40, 58, 0.5) 100%);
    border-radius: 14px;
    border: 1px solid rgba(137, 180, 250, 0.2);
    padding: 16px;
    margin: 12px;
}

.media-widget .album-art {
    border-radius: 10px;
    background: linear-gradient(135deg, rgba(30, 30, 46, 0.8) 0%, rgba(24, 24, 37, 0.6) 100%);
    min-width: 72px;
    min-height: 72px;
    border: 1px solid rgba(69, 71, 90, 0.3);
}

.media-widget .media-title {
    font-size: 15px;
    font-weight: 700;
    color: #cdd6f4;
    letter-spacing: 0.3px;
}

.media-widget .media-artist {
    font-size: 13px;
    color: rgba(166, 227, 161, 0.9);
    font-weight: 500;
}

.media-widget .media-controls {
    margin-left: 12px;
}

.media-widget .media-button {
    background: rgba(69, 71, 90, 0.3);
    border: 1px solid rgba(69, 71, 90, 0.3);
    border-radius: 50%;
    min-width: 38px;
    min-height: 38px;
    padding: 8px;
    color: rgba(205, 214, 244, 0.9);
    transition: all 0.2s ease;
}

.media-widget .media-button:hover {
    background: rgba(137, 180, 250, 0.25);
    border-color: rgba(137, 180, 250, 0.4);
    color: #89b4fa;
}

.media-widget .media-button.play-pause {
    min-width: 46px;
    min-height: 46px;
    background: linear-gradient(135deg, rgba(137, 180, 250, 0.25) 0%, rgba(137, 180, 250, 0.15) 100%);
    border: 1px solid rgba(137, 180, 250, 0.3);
}

.media-widget .media-button.play-pause:hover {
    background: linear-gradient(135deg, rgba(137, 180, 250, 0.4) 0%, rgba(137, 180, 250, 0.25) 100%);
    border-color: rgba(137, 180, 250, 0.5);
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
