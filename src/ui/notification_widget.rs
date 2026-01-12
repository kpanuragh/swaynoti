use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Image, Label, Orientation, ProgressBar, Widget};
use tracing::debug;

use crate::config::Config;
use crate::notification::Notification;

/// Widget for displaying a single notification
pub struct NotificationWidget {
    container: GtkBox,
}

impl NotificationWidget {
    pub fn new(notification: &Notification, config: &Config) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 12);
        container.add_css_class("notification");
        container.add_css_class(notification.hints.urgency.css_class());

        // Icon (left side)
        if let Some(icon) = Self::create_icon(notification, config) {
            container.append(&icon);
        }

        // Content (right side)
        let content_box = GtkBox::new(Orientation::Vertical, 4);
        content_box.set_hexpand(true);

        // App name (optional)
        if config.appearance.show_app_name && !notification.app_name.is_empty() {
            let app_label = Label::new(Some(&notification.app_name));
            app_label.add_css_class("app-name");
            app_label.set_halign(Align::Start);
            app_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            content_box.append(&app_label);
        }

        // Summary (title)
        let summary_label = Label::new(Some(&notification.summary));
        summary_label.add_css_class("summary");
        summary_label.set_halign(Align::Start);
        summary_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        summary_label.set_max_width_chars(40);
        content_box.append(&summary_label);

        // Body
        if !notification.body.is_empty() {
            let body_label = Label::new(Some(&notification.body));
            body_label.add_css_class("body");
            body_label.set_halign(Align::Start);
            body_label.set_wrap(true);
            body_label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            body_label.set_max_width_chars(45);
            body_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            body_label.set_lines(3);

            // Enable markup if configured
            if config.general.markup {
                body_label.set_use_markup(true);
            }

            content_box.append(&body_label);
        }

        // Progress bar (if present)
        if let Some(progress) = notification.progress() {
            let progress_bar = ProgressBar::new();
            progress_bar.add_css_class("progress");
            progress_bar.set_fraction(progress as f64 / 100.0);
            progress_bar.set_margin_top(8);
            content_box.append(&progress_bar);
        }

        // Action buttons
        if !notification.actions.is_empty() {
            let actions_box = Self::create_actions(notification);
            content_box.append(&actions_box);
        }

        container.append(&content_box);

        // Close button
        let close_button = Button::new();
        close_button.add_css_class("close-button");
        close_button.set_icon_name("window-close-symbolic");
        close_button.set_valign(Align::Start);
        close_button.set_tooltip_text(Some("Dismiss"));
        container.append(&close_button);

        Self { container }
    }

    /// Create the notification icon
    fn create_icon(notification: &Notification, config: &Config) -> Option<Widget> {
        let size = config.appearance.icon_size as i32;

        // Try image data from hints first
        if let Some(ref image_data) = notification.hints.image_data {
            // Create pixbuf from raw data
            let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
                &glib::Bytes::from(&image_data.data),
                gdk_pixbuf::Colorspace::Rgb,
                image_data.has_alpha,
                image_data.bits_per_sample,
                image_data.width,
                image_data.height,
                image_data.rowstride,
            );
            if let Some(scaled) = pixbuf.scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
            {
                let texture = gtk4::gdk::Texture::for_pixbuf(&scaled);
                let image = Image::from_paintable(Some(&texture));
                image.add_css_class("icon");
                return Some(image.upcast());
            }
        }

        // Try image path from hints
        if let Some(ref path) = notification.hints.image_path {
            let image = Image::from_file(path);
            image.set_pixel_size(size);
            image.add_css_class("icon");
            return Some(image.upcast());
        }

        // Try app_icon
        if !notification.app_icon.is_empty() {
            let icon = &notification.app_icon;

            // Check if it's a file path
            if icon.starts_with('/') || icon.starts_with("file://") {
                let path = icon.strip_prefix("file://").unwrap_or(icon);
                let image = Image::from_file(path);
                image.set_pixel_size(size);
                image.add_css_class("icon");
                return Some(image.upcast());
            }

            // Treat as icon name
            let image = Image::from_icon_name(icon);
            image.set_pixel_size(size);
            image.add_css_class("icon");
            return Some(image.upcast());
        }

        // No icon
        None
    }

    /// Create action buttons
    fn create_actions(notification: &Notification) -> GtkBox {
        let actions_box = GtkBox::new(Orientation::Horizontal, 6);
        actions_box.add_css_class("actions");
        actions_box.set_margin_top(8);

        for (key, label) in &notification.actions {
            // Skip default action (handled by clicking notification)
            if key == "default" {
                continue;
            }

            let button = Button::with_label(label);
            button.add_css_class("action-button");

            let action_key = key.clone();
            let notification_id = notification.id;

            button.connect_clicked(move |_| {
                debug!(
                    "Action '{}' clicked on notification {}",
                    action_key, notification_id
                );
                // Action will be handled via callback
            });

            actions_box.append(&button);
        }

        actions_box
    }

    /// Get the container widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Update the notification content
    pub fn update(&self, notification: &Notification, _config: &Config) {
        // For now, just update CSS classes
        self.container.remove_css_class("low");
        self.container.remove_css_class("normal");
        self.container.remove_css_class("critical");
        self.container
            .add_css_class(notification.hints.urgency.css_class());
    }
}
