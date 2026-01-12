#!/bin/bash
set -e

# Swaynoti Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/swaynoti/swaynoti/main/install.sh | bash

VERSION="${SWAYNOTI_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/swaynoti"
SYSTEMD_USER_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac
}

# Get latest version from GitHub
get_latest_version() {
    curl -sL "https://api.github.com/repos/swaynoti/swaynoti/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"v([^"]+)".*/\1/'
}

# Check dependencies
check_dependencies() {
    info "Checking dependencies..."

    local missing=()

    # Check for GTK4
    if ! pkg-config --exists gtk4 2>/dev/null; then
        missing+=("libgtk-4-dev (Debian/Ubuntu) or gtk4-devel (Fedora)")
    fi

    # Check for layer-shell
    if ! pkg-config --exists gtk4-layer-shell-0 2>/dev/null; then
        missing+=("libgtk4-layer-shell-dev (Debian/Ubuntu) or gtk4-layer-shell-devel (Fedora)")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        warn "Missing dependencies:"
        for dep in "${missing[@]}"; do
            echo "  - $dep"
        done
        echo ""
        echo "Install dependencies first:"
        echo "  Debian/Ubuntu: sudo apt install libgtk-4-1 libgtk4-layer-shell0"
        echo "  Fedora: sudo dnf install gtk4 gtk4-layer-shell"
        echo ""
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Download and install
install_swaynoti() {
    local arch=$(detect_arch)

    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_version)
        if [ -z "$VERSION" ]; then
            error "Failed to get latest version"
        fi
    fi

    info "Installing swaynoti v${VERSION} for ${arch}..."

    local download_url="https://github.com/swaynoti/swaynoti/releases/download/v${VERSION}/swaynoti-linux-${arch}.tar.gz"
    local tmp_dir=$(mktemp -d)

    info "Downloading from ${download_url}..."
    curl -fsSL "$download_url" -o "${tmp_dir}/swaynoti.tar.gz" || \
        error "Failed to download swaynoti"

    info "Extracting..."
    tar -xzf "${tmp_dir}/swaynoti.tar.gz" -C "${tmp_dir}"

    # Install binaries
    info "Installing binaries to ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ]; then
        cp "${tmp_dir}/swaynoti" "${tmp_dir}/swaynotictl" "$INSTALL_DIR/"
    else
        sudo cp "${tmp_dir}/swaynoti" "${tmp_dir}/swaynotictl" "$INSTALL_DIR/"
    fi
    chmod +x "${INSTALL_DIR}/swaynoti" "${INSTALL_DIR}/swaynotictl"

    # Install systemd user service
    info "Installing systemd user service..."
    mkdir -p "$SYSTEMD_USER_DIR"
    cp "${tmp_dir}/systemd/swaynoti.service" "$SYSTEMD_USER_DIR/"

    # Install D-Bus service (user session)
    info "Installing D-Bus service..."
    local dbus_dir="${XDG_DATA_HOME:-$HOME/.local/share}/dbus-1/services"
    mkdir -p "$dbus_dir"
    cat > "${dbus_dir}/org.freedesktop.Notifications.service" << EOF
[D-BUS Service]
Name=org.freedesktop.Notifications
Exec=${INSTALL_DIR}/swaynoti
EOF

    # Install default config
    info "Installing default configuration..."
    mkdir -p "$CONFIG_DIR"
    if [ ! -f "${CONFIG_DIR}/config.toml" ]; then
        cp "${tmp_dir}/config/default.toml" "${CONFIG_DIR}/config.toml"
    else
        warn "Config already exists, skipping"
    fi

    if [ -d "${tmp_dir}/config/themes" ]; then
        cp -r "${tmp_dir}/config/themes" "$CONFIG_DIR/"
    fi

    # Cleanup
    rm -rf "$tmp_dir"

    info "Installation complete!"
    echo ""
    echo "To start swaynoti:"
    echo "  1. Kill any existing notification daemon (mako, dunst, etc.):"
    echo "     pkill mako; pkill dunst"
    echo ""
    echo "  2. Enable and start swaynoti:"
    echo "     systemctl --user daemon-reload"
    echo "     systemctl --user enable --now swaynoti.service"
    echo ""
    echo "Configuration file: ${CONFIG_DIR}/config.toml"
}

# Uninstall
uninstall_swaynoti() {
    info "Uninstalling swaynoti..."

    # Stop service
    systemctl --user stop swaynoti.service 2>/dev/null || true
    systemctl --user disable swaynoti.service 2>/dev/null || true

    # Remove binaries
    if [ -w "$INSTALL_DIR" ]; then
        rm -f "${INSTALL_DIR}/swaynoti" "${INSTALL_DIR}/swaynotictl"
    else
        sudo rm -f "${INSTALL_DIR}/swaynoti" "${INSTALL_DIR}/swaynotictl"
    fi

    # Remove service files
    rm -f "${SYSTEMD_USER_DIR}/swaynoti.service"
    rm -f "${XDG_DATA_HOME:-$HOME/.local/share}/dbus-1/services/org.freedesktop.Notifications.service"

    info "Uninstallation complete!"
    echo "Note: Configuration files in ${CONFIG_DIR} were not removed."
}

# Main
case "${1:-install}" in
    install)
        check_dependencies
        install_swaynoti
        ;;
    uninstall)
        uninstall_swaynoti
        ;;
    *)
        echo "Usage: $0 [install|uninstall]"
        exit 1
        ;;
esac
