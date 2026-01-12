# Swaynoti

A modern, lightweight notification daemon for Wayland compositors.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Wayland](https://img.shields.io/badge/wayland-compatible-green.svg)

## Features

- **Full FreeDesktop Notifications Specification** support
- **Native Wayland** support via GTK4 + layer-shell
- **Works with all Wayland compositors**: Sway, Hyprland, River, and more
- **TOML configuration** with CSS theming
- **Per-application rules** with regex matching
- **Do Not Disturb** mode with scheduling
- **Notification history** (coming soon)
- **Multi-monitor support**
- **IPC control** via `swaynotictl`

## Installation

### From Releases (Recommended)

Download the latest release from the [Releases page](https://github.com/swaynoti/swaynoti/releases).

**Debian/Ubuntu:**
```bash
sudo dpkg -i swaynoti_*_amd64.deb
sudo apt-get install -f  # Install dependencies if needed
```

**Fedora/RHEL/CentOS:**
```bash
sudo dnf install swaynoti-*.x86_64.rpm
```

**Manual Installation:**
```bash
curl -fsSL https://raw.githubusercontent.com/swaynoti/swaynoti/main/install.sh | bash
```

### From Source

#### Dependencies

**Debian/Ubuntu:**
```bash
sudo apt install libgtk-4-dev libgraphene-1.0-dev libgtk4-layer-shell-dev pkg-config
```

**Fedora:**
```bash
sudo dnf install gtk4-devel graphene-devel gtk4-layer-shell-devel pkg-config
```

**Arch Linux:**
```bash
sudo pacman -S gtk4 graphene gtk4-layer-shell
```

#### Building

```bash
git clone https://github.com/swaynoti/swaynoti.git
cd swaynoti
cargo build --release

# Install
sudo cp target/release/swaynoti target/release/swaynotictl /usr/local/bin/
```

## Usage

### Starting the Daemon

First, stop any existing notification daemon:
```bash
pkill mako
pkill dunst
pkill swaync
```

Then start swaynoti:
```bash
# Using systemd (recommended)
systemctl --user enable --now swaynoti.service

# Or manually
swaynoti &
```

### Configuration

Configuration file location: `~/.config/swaynoti/config.toml`

```toml
[general]
max_visible = 5
sort_order = "newest-first"
markup = true

[appearance]
width = 350
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
anchor = "top-right"  # top-left, top-center, top-right, bottom-left, bottom-center, bottom-right
layer = "overlay"
stacking = "vertical"

[positioning.margin]
top = 10
right = 10
bottom = 10
left = 10

[timeouts]
default = 5000
low = 3000
normal = 5000
critical = 0  # 0 = never expires

[history]
enabled = true
max_entries = 100

# Per-app rules
[[rules]]
[rules.criteria]
app_name = "Spotify"
[rules.actions]
timeout = 3000
css_class = "spotify"
skip_history = true
```

### CSS Theming

Custom themes can be applied via CSS. Create `~/.config/swaynoti/theme.css`:

```css
.notification {
    background-color: rgba(30, 30, 46, 0.95);
    border-radius: 12px;
    padding: 12px;
    border: 1px solid rgba(69, 71, 90, 0.5);
}

.notification.critical {
    border-left: 4px solid #f38ba8;
}

.notification .summary {
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
}

.notification .body {
    font-size: 13px;
    color: rgba(205, 214, 244, 0.85);
}
```

Then reference it in your config:
```toml
[appearance]
theme = "~/.config/swaynoti/theme.css"
```

### Control Commands

```bash
# List active notifications
swaynotictl list

# Get notification count
swaynotictl count

# Dismiss a notification
swaynotictl dismiss <id>

# Dismiss all notifications
swaynotictl dismiss-all

# Toggle Do Not Disturb
swaynotictl toggle-dnd

# Check DND status
swaynotictl dnd-status
```

### Waybar Integration

Add to your waybar config:
```json
"custom/notifications": {
    "exec": "swaynotictl count",
    "interval": 1,
    "format": " {}",
    "on-click": "swaynotictl toggle-dnd"
}
```

## Sway Configuration

Add to `~/.config/sway/config`:
```
exec swaynoti
```

Or use systemd:
```
exec systemctl --user start swaynoti.service
```

## Hyprland Configuration

Add to `~/.config/hypr/hyprland.conf`:
```
exec-once = swaynoti
```

## Troubleshooting

### Notifications not appearing

1. Check if another notification daemon is running:
   ```bash
   busctl --user list | grep Notifications
   ```

2. Kill the conflicting daemon:
   ```bash
   pkill mako
   systemctl --user stop mako.service
   ```

3. Disable D-Bus activation for other daemons:
   ```bash
   sudo mv /usr/share/dbus-1/services/fr.emersion.mako.service{,.disabled}
   ```

### Debug mode

Run with debug logging:
```bash
swaynoti -d
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgements

- [mako](https://github.com/emersion/mako) - Inspiration and reference implementation
- [dunst](https://github.com/dunst-project/dunst) - FreeDesktop notification spec reference
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) - Wayland layer-shell support
