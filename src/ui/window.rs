
use async_channel::Sender;
use gtk4::prelude::*;
use gtk4::{Application, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tracing::{debug, info};

use crate::config::{Anchor, Config};
use crate::notification::{ActionEvent, Notification};

use super::NotificationWidget;

/// A layer-shell window displaying a single notification
pub struct NotificationWindow {
    window: Window,
    notification_id: u32,
    widget: NotificationWidget,
    #[allow(dead_code)]
    action_sender: Sender<ActionEvent>,
}

impl NotificationWindow {
    /// Create a new notification window
    pub fn new(
        app: &Application,
        notification: &Notification,
        config: &Config,
        index: usize,
        action_sender: Sender<ActionEvent>,
    ) -> Self {
        let window = Window::builder()
            .application(app)
            .decorated(false)
            .resizable(false)
            .build();

        // Initialize layer shell
        window.init_layer_shell();

        // Set the layer
        let layer = match config.positioning.layer {
            crate::config::Layer::Background => Layer::Background,
            crate::config::Layer::Bottom => Layer::Bottom,
            crate::config::Layer::Top => Layer::Top,
            crate::config::Layer::Overlay => Layer::Overlay,
        };
        window.set_layer(layer);

        // Set anchors based on config
        Self::apply_anchors(&window, &config.positioning.anchor);

        // Set margins
        Self::apply_margins(&window, config, index);

        // No keyboard focus
        window.set_keyboard_mode(KeyboardMode::None);

        // Don't reserve screen space
        window.set_exclusive_zone(0);

        // Set width
        window.set_default_width(config.appearance.width as i32);

        // Create notification widget
        let widget = NotificationWidget::new(notification, config);
        window.set_child(Some(widget.widget()));

        // Add window CSS class
        window.add_css_class("notification-window");

        // Setup event handlers
        let sender = action_sender.clone();
        let id = notification.id;
        Self::setup_event_handlers(&window, id, sender);

        Self {
            window,
            notification_id: notification.id,
            widget,
            action_sender,
        }
    }

    /// Apply anchor positions based on config
    fn apply_anchors(window: &Window, anchor: &Anchor) {
        // Reset all anchors
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        match anchor {
            Anchor::TopLeft => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Left, true);
            }
            Anchor::TopCenter => {
                window.set_anchor(Edge::Top, true);
            }
            Anchor::TopRight => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
            }
            Anchor::BottomLeft => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
            }
            Anchor::BottomCenter => {
                window.set_anchor(Edge::Bottom, true);
            }
            Anchor::BottomRight => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Right, true);
            }
        }
    }

    /// Apply margins based on config and stacking index
    fn apply_margins(window: &Window, config: &Config, index: usize) {
        let margin = &config.positioning.margin;
        let gap = config.appearance.gap as i32;
        let estimated_height = 100; // Approximate notification height

        // Calculate stacking offset
        let stack_offset = (index as i32) * (estimated_height + gap);

        match config.positioning.anchor {
            Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight => {
                window.set_margin(Edge::Top, margin.top + stack_offset);
                window.set_margin(Edge::Bottom, margin.bottom);
            }
            Anchor::BottomLeft | Anchor::BottomCenter | Anchor::BottomRight => {
                window.set_margin(Edge::Top, margin.top);
                window.set_margin(Edge::Bottom, margin.bottom + stack_offset);
            }
        }

        window.set_margin(Edge::Left, margin.left);
        window.set_margin(Edge::Right, margin.right);
    }

    /// Setup event handlers for the window
    fn setup_event_handlers(window: &Window, id: u32, sender: Sender<ActionEvent>) {
        // Click handler (dismiss or default action)
        let click = gtk4::GestureClick::new();
        let sender_click = sender.clone();
        click.connect_released(move |gesture, _, _, _| {
            if gesture.current_button() == gtk4::gdk::BUTTON_PRIMARY {
                debug!("Notification {} clicked", id);
                let sender = sender_click.clone();
                glib::spawn_future_local(async move {
                    let _ = sender.send(ActionEvent::Dismissed { id }).await;
                });
            }
        });
        window.add_controller(click);

        // Right-click handler (context menu)
        let right_click = gtk4::GestureClick::new();
        right_click.set_button(gtk4::gdk::BUTTON_SECONDARY);
        right_click.connect_released(move |_, _, x, y| {
            debug!("Right-click on notification {} at ({}, {})", id, x, y);
            // TODO: Show context menu
        });
        window.add_controller(right_click);

        // Hover handlers
        let motion = gtk4::EventControllerMotion::new();
        let sender_enter = sender.clone();
        let sender_leave = sender.clone();

        motion.connect_enter(move |_, _, _| {
            debug!("Mouse entered notification {}", id);
            let sender = sender_enter.clone();
            glib::spawn_future_local(async move {
                let _ = sender.send(ActionEvent::Hovered { id }).await;
            });
        });

        motion.connect_leave(move |_| {
            debug!("Mouse left notification {}", id);
            let sender = sender_leave.clone();
            glib::spawn_future_local(async move {
                let _ = sender.send(ActionEvent::Unhovered { id }).await;
            });
        });

        window.add_controller(motion);
    }

    /// Show the window
    pub fn show(&self) {
        self.window.present();
        info!(
            "Showing notification window for id={}",
            self.notification_id
        );
    }

    /// Hide and destroy the window
    pub fn close(&self) {
        self.window.close();
        debug!("Closed notification window for id={}", self.notification_id);
    }

    /// Update the notification content
    pub fn update(&self, notification: &Notification, config: &Config) {
        self.widget.update(notification, config);
    }

    /// Update the window position (for reordering)
    pub fn update_position(&self, config: &Config, index: usize) {
        Self::apply_margins(&self.window, config, index);
    }

    /// Get the notification ID
    pub fn id(&self) -> u32 {
        self.notification_id
    }

    /// Get the underlying GTK window
    pub fn window(&self) -> &Window {
        &self.window
    }
}
