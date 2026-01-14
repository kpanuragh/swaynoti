use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::Sender;
use glib::SourceId;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Image, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow,
    Separator, Window,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use parking_lot::RwLock;
use tracing::{debug, info};

use crate::config::Config;
use crate::history::{HistoryEntry, HistoryStore};
use crate::notification::ActionEvent;

use super::media_widget::MediaWidget;

/// Notification center panel showing history
pub struct NotificationCenter {
    window: Window,
    list_box: ListBox,
    config: Arc<RwLock<Config>>,
    history_store: Option<Arc<HistoryStore>>,
    action_sender: Sender<ActionEvent>,
    visible: bool,
    media_widget: Rc<RefCell<MediaWidget>>,
    refresh_timer: Rc<RefCell<Option<SourceId>>>,
}

impl NotificationCenter {
    /// Create a new notification center
    pub fn new(
        app: &gtk4::Application,
        config: Arc<RwLock<Config>>,
        history_store: Option<Arc<HistoryStore>>,
        action_sender: Sender<ActionEvent>,
    ) -> Self {
        let window = Window::builder()
            .application(app)
            .title("Notification Center")
            .decorated(false)
            .resizable(false)
            .build();

        // Set width (height will be full screen via anchoring)
        window.set_size_request(380, -1);

        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_exclusive_zone(0);

        // Anchor to top, right, and bottom for full height
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Bottom, true);

        // Set margins
        window.set_margin(Edge::Top, 10);
        window.set_margin(Edge::Right, 10);
        window.set_margin(Edge::Bottom, 10);

        // Add CSS class
        window.add_css_class("notification-center");

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.add_css_class("notification-center-container");

        // Header
        let header = Self::create_header();
        main_box.append(&header);

        // Separator
        let sep = Separator::new(Orientation::Horizontal);
        main_box.append(&sep);

        // Media player widget
        let media_widget = Rc::new(RefCell::new(MediaWidget::new()));
        main_box.append(media_widget.borrow().widget());

        // Scrolled list of notifications
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        list_box.add_css_class("notification-list");

        scrolled.set_child(Some(&list_box));
        main_box.append(&scrolled);

        // Footer with clear button
        let footer = Self::create_footer(history_store.clone(), list_box.clone());
        main_box.append(&footer);

        window.set_child(Some(&main_box));

        // Close on click outside (Escape key)
        let window_clone = window.clone();
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gtk4::gdk::Key::Escape {
                window_clone.set_visible(false);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(key_controller);

        let mut center = Self {
            window,
            list_box,
            config,
            history_store,
            action_sender,
            visible: false,
            media_widget,
            refresh_timer: Rc::new(RefCell::new(None)),
        };

        center.refresh();
        center
    }

    /// Create header section
    fn create_header() -> GtkBox {
        let header = GtkBox::new(Orientation::Horizontal, 8);
        header.add_css_class("notification-center-header");
        header.set_margin_start(16);
        header.set_margin_end(16);
        header.set_margin_top(12);
        header.set_margin_bottom(12);

        let title = Label::new(Some("Notifications"));
        title.add_css_class("notification-center-title");
        title.set_hexpand(true);
        title.set_halign(Align::Start);

        header.append(&title);

        header
    }

    /// Create footer section with clear button
    fn create_footer(history_store: Option<Arc<HistoryStore>>, list_box: ListBox) -> GtkBox {
        let footer = GtkBox::new(Orientation::Horizontal, 8);
        footer.add_css_class("notification-center-footer");
        footer.set_margin_start(16);
        footer.set_margin_end(16);
        footer.set_margin_top(12);
        footer.set_margin_bottom(12);

        let clear_btn = Button::with_label("Clear All");
        clear_btn.add_css_class("clear-all-button");
        clear_btn.set_halign(Align::End);
        clear_btn.set_hexpand(true);

        // Connect click handler
        clear_btn.connect_clicked(move |_| {
            debug!("Clear All button clicked");

            // Clear history from database
            if let Some(ref store) = history_store {
                if let Err(e) = store.clear() {
                    debug!("Failed to clear history: {}", e);
                }
            }

            // Clear the list box
            while let Some(row) = list_box.first_child() {
                list_box.remove(&row);
            }

            // Add empty message
            let row = ListBoxRow::new();
            row.set_selectable(false);
            let label = Label::new(Some("No notifications"));
            label.add_css_class("empty-message");
            label.set_margin_top(40);
            label.set_margin_bottom(40);
            row.set_child(Some(&label));
            list_box.append(&row);

            info!("Notification history cleared");
        });

        footer.append(&clear_btn);

        footer
    }

    /// Refresh the notification list from history
    pub fn refresh(&mut self) {
        debug!("Refreshing notification center");

        // Refresh media widget
        self.media_widget.borrow().refresh();

        // Clear existing items
        while let Some(row) = self.list_box.first_child() {
            self.list_box.remove(&row);
        }

        let Some(ref store) = self.history_store else {
            debug!("No history store available");
            self.add_empty_message();
            return;
        };

        // Get grouped history
        match store.get_grouped(100) {
            Ok(groups) => {
                debug!("Got {} app groups from history", groups.len());
                if groups.is_empty() {
                    self.add_empty_message();
                    return;
                }

                for (app_name, entries) in groups {
                    debug!("Adding group: {} with {} entries", app_name, entries.len());
                    self.add_app_group(&app_name, &entries);
                }
            }
            Err(e) => {
                debug!("Failed to get history: {}", e);
                self.add_empty_message();
            }
        }
    }

    /// Add empty message when no notifications
    fn add_empty_message(&self) {
        let row = ListBoxRow::new();
        row.set_selectable(false);

        let label = Label::new(Some("No notifications"));
        label.add_css_class("empty-message");
        label.set_margin_top(40);
        label.set_margin_bottom(40);

        row.set_child(Some(&label));
        self.list_box.append(&row);
    }

    /// Add an app group to the list
    fn add_app_group(&self, app_name: &str, entries: &[HistoryEntry]) {
        // App header row
        let header_row = ListBoxRow::new();
        header_row.set_selectable(false);

        let header_box = GtkBox::new(Orientation::Horizontal, 8);
        header_box.add_css_class("app-group-header");
        header_box.set_margin_start(8);
        header_box.set_margin_end(8);
        header_box.set_margin_top(4);
        header_box.set_margin_bottom(4);

        // App icon
        let icon = Image::from_icon_name(app_name);
        icon.set_pixel_size(24);
        icon.add_css_class("app-icon");
        header_box.append(&icon);

        // App name
        let name_label = Label::new(Some(app_name));
        name_label.add_css_class("app-name");
        name_label.set_hexpand(true);
        name_label.set_halign(Align::Start);
        header_box.append(&name_label);

        // Count badge
        let count_label = Label::new(Some(&format!("{}", entries.len())));
        count_label.add_css_class("notification-count");
        header_box.append(&count_label);

        header_row.set_child(Some(&header_box));
        self.list_box.append(&header_row);

        // Notification entries
        for entry in entries.iter().take(5) {
            // Show max 5 per app
            self.add_notification_entry(entry);
        }

        // Show "and X more" if there are more
        if entries.len() > 5 {
            let more_row = ListBoxRow::new();
            more_row.set_selectable(false);

            let more_label = Label::new(Some(&format!("and {} more...", entries.len() - 5)));
            more_label.add_css_class("more-notifications");
            more_label.set_margin_start(36);
            more_label.set_margin_top(4);
            more_label.set_margin_bottom(8);
            more_label.set_halign(Align::Start);

            more_row.set_child(Some(&more_label));
            self.list_box.append(&more_row);
        }
    }

    /// Add a single notification entry
    fn add_notification_entry(&self, entry: &HistoryEntry) {
        let row = ListBoxRow::new();
        row.set_selectable(false);

        let entry_box = GtkBox::new(Orientation::Vertical, 2);
        entry_box.add_css_class("notification-entry");
        entry_box.set_margin_start(32);
        entry_box.set_margin_end(8);
        entry_box.set_margin_top(2);
        entry_box.set_margin_bottom(2);

        // Summary
        let summary = Label::new(Some(&entry.summary));
        summary.add_css_class("entry-summary");
        summary.set_halign(Align::Start);
        summary.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        summary.set_max_width_chars(40);
        entry_box.append(&summary);

        // Body (if present)
        if !entry.body.is_empty() {
            let body = Label::new(Some(&entry.body));
            body.add_css_class("entry-body");
            body.set_halign(Align::Start);
            body.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            body.set_max_width_chars(45);
            entry_box.append(&body);
        }

        // Timestamp
        let time_ago = Self::format_time_ago(&entry.timestamp);
        let time_label = Label::new(Some(&time_ago));
        time_label.add_css_class("entry-time");
        time_label.set_halign(Align::Start);
        entry_box.append(&time_label);

        row.set_child(Some(&entry_box));
        self.list_box.append(&row);
    }

    /// Format timestamp as "X minutes ago", "X hours ago", etc.
    fn format_time_ago(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(*timestamp);

        if duration.num_minutes() < 1 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            let mins = duration.num_minutes();
            format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if duration.num_hours() < 24 {
            let hours = duration.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else {
            let days = duration.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        }
    }

    /// Show the notification center
    pub fn show(&mut self) {
        self.refresh();
        self.window.present();
        self.visible = true;

        // Start periodic refresh timer for media widget (every 2 seconds)
        self.start_refresh_timer();

        info!("Notification center shown");
    }

    /// Hide the notification center
    pub fn hide(&mut self) {
        // Stop refresh timer
        self.stop_refresh_timer();

        self.window.set_visible(false);
        self.visible = false;
        info!("Notification center hidden");
    }

    /// Start the media widget refresh timer
    fn start_refresh_timer(&self) {
        let media_widget = self.media_widget.clone();
        let timer_holder = self.refresh_timer.clone();

        let source_id = glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
            media_widget.borrow().refresh();
            glib::ControlFlow::Continue
        });

        *timer_holder.borrow_mut() = Some(source_id);
        debug!("Media refresh timer started");
    }

    /// Stop the media widget refresh timer
    fn stop_refresh_timer(&self) {
        if let Some(source_id) = self.refresh_timer.borrow_mut().take() {
            source_id.remove();
            debug!("Media refresh timer stopped");
        }
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Clear all history
    pub fn clear_all(&mut self) {
        if let Some(ref store) = self.history_store {
            if let Err(e) = store.clear() {
                debug!("Failed to clear history: {}", e);
            }
        }
        self.refresh();
    }

    /// Get window reference
    pub fn window(&self) -> &Window {
        &self.window
    }
}
