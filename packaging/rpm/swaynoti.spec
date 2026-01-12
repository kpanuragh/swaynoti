Name:           swaynoti
Version:        @VERSION@
Release:        1%{?dist}
Summary:        Modern Wayland notification daemon

License:        MIT
URL:            https://github.com/swaynoti/swaynoti
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64

Requires:       gtk4 >= 4.6
Requires:       graphene
Requires:       gtk4-layer-shell

%description
Swaynoti is a lightweight, highly customizable notification daemon for
Wayland compositors like Sway, Hyprland, River, and others.

Features:
- Full FreeDesktop Notifications specification support
- GTK4 with layer-shell for native Wayland integration
- TOML configuration with CSS theming
- Per-application rules
- Do Not Disturb mode
- Notification history

%prep
%setup -q

%install
rm -rf %{buildroot}

# Create directories
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_userunitdir}
mkdir -p %{buildroot}%{_datadir}/dbus-1/services
mkdir -p %{buildroot}%{_sysconfdir}/swaynoti
mkdir -p %{buildroot}%{_docdir}/%{name}

# Install binaries
install -m 755 swaynoti %{buildroot}%{_bindir}/swaynoti
install -m 755 swaynotictl %{buildroot}%{_bindir}/swaynotictl

# Install systemd user service
install -m 644 systemd/swaynoti.service %{buildroot}%{_userunitdir}/swaynoti.service

# Install D-Bus service
install -m 644 systemd/org.freedesktop.Notifications.service %{buildroot}%{_datadir}/dbus-1/services/

# Install config
install -m 644 config/default.toml %{buildroot}%{_sysconfdir}/swaynoti/config.toml
cp -r config/themes %{buildroot}%{_sysconfdir}/swaynoti/

%files
%{_bindir}/swaynoti
%{_bindir}/swaynotictl
%{_userunitdir}/swaynoti.service
%{_datadir}/dbus-1/services/org.freedesktop.Notifications.service
%config(noreplace) %{_sysconfdir}/swaynoti/config.toml
%{_sysconfdir}/swaynoti/themes/

%post
echo "Swaynoti installed successfully!"
echo ""
echo "To start swaynoti:"
echo "  systemctl --user enable --now swaynoti.service"
echo ""
echo "Configuration: %{_sysconfdir}/swaynoti/config.toml"
echo "User config:   ~/.config/swaynoti/config.toml"

%changelog
* %(date "+%a %b %d %Y") Swaynoti Developers <kpanuragh@gmail.com> - @VERSION@-1
- Initial package release
