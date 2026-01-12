use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use gtk4::prelude::*;
use gtk4::{gio, Application};
use parking_lot::RwLock;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::notification::{ActionEvent, Notification, UiEvent};

use super::window::NotificationWindow;
use super::style::StyleManager;

const APP_ID: &str = "org.swaynoti.daemon";

/// Main GTK4 application for swaynoti
pub struct SwaynotiApp {
    app: Application,
    config: Arc<RwLock<Config>>,
    style_manager: Rc<StyleManager>,
    windows: Rc<RefCell<HashMap<u32, NotificationWindow>>>,
    action_sender: Sender<ActionEvent>,
}

impl SwaynotiApp {
    /// Create a new swaynoti application
    pub fn new(
        config: Arc<RwLock<Config>>,
        action_sender: Sender<ActionEvent>,
    ) -> Self {
        let app = Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::NON_UNIQUE)
            .build();

        let style_manager = Rc::new(StyleManager::new(config.clone()));
        let windows = Rc::new(RefCell::new(HashMap::new()));

        Self {
            app,
            config,
            style_manager,
            windows,
            action_sender,
        }
    }

    /// Run the application with the UI event receiver
    pub fn run(self, ui_receiver: Receiver<UiEvent>) {
        // Load styles
        self.style_manager.load_styles();

        // Get references for the event loop
        let app = self.app.clone();
        let config = self.config.clone();
        let windows = self.windows.clone();
        let action_sender = self.action_sender.clone();

        // Spawn UI event handler on GLib main context
        glib::MainContext::default().spawn_local(async move {
            Self::handle_ui_events(
                app,
                config,
                windows,
                action_sender,
                ui_receiver,
            ).await;
        });

        // Run the GLib main loop (this blocks)
        info!("Starting GLib main loop");
        let main_loop = glib::MainLoop::new(None, false);
        main_loop.run();
    }

    /// Handle UI events from the notification manager
    async fn handle_ui_events(
        app: Application,
        config: Arc<RwLock<Config>>,
        windows: Rc<RefCell<HashMap<u32, NotificationWindow>>>,
        action_sender: Sender<ActionEvent>,
        receiver: Receiver<UiEvent>,
    ) {
        info!("UI event handler started");

        while let Ok(event) = receiver.recv().await {
            match event {
                UiEvent::Show(notification) => {
                    Self::show_notification(
                        &app,
                        &config,
                        &windows,
                        &action_sender,
                        notification,
                    );
                }
                UiEvent::Update(id, notification) => {
                    Self::update_notification(&config, &windows, id, notification);
                }
                UiEvent::Close(id) => {
                    Self::close_notification(&windows, id);
                }
                UiEvent::Reposition => {
                    Self::reposition_all(&config, &windows);
                }
            }
        }

        info!("UI event handler stopped");
    }

    /// Show a new notification
    fn show_notification(
        app: &Application,
        config: &Arc<RwLock<Config>>,
        windows: &Rc<RefCell<HashMap<u32, NotificationWindow>>>,
        action_sender: &Sender<ActionEvent>,
        notification: Notification,
    ) {
        let id = notification.id;
        let config_read = config.read();

        // Calculate index for stacking
        let index = windows.borrow().len();

        // Check max visible limit
        if index >= config_read.general.max_visible as usize {
            debug!("Max visible notifications reached, not showing {}", id);
            return;
        }

        let window = NotificationWindow::new(
            app,
            &notification,
            &config_read,
            index,
            action_sender.clone(),
        );

        window.show();
        windows.borrow_mut().insert(id, window);

        info!("Displayed notification {} (total: {})", id, windows.borrow().len());
    }

    /// Update an existing notification
    fn update_notification(
        config: &Arc<RwLock<Config>>,
        windows: &Rc<RefCell<HashMap<u32, NotificationWindow>>>,
        id: u32,
        notification: Notification,
    ) {
        let windows_ref = windows.borrow();
        if let Some(window) = windows_ref.get(&id) {
            let config_read = config.read();
            window.update(&notification, &config_read);
            debug!("Updated notification {}", id);
        }
    }

    /// Close a notification
    fn close_notification(
        windows: &Rc<RefCell<HashMap<u32, NotificationWindow>>>,
        id: u32,
    ) {
        let window = windows.borrow_mut().remove(&id);
        if let Some(window) = window {
            window.close();
            info!("Closed notification {} (remaining: {})", id, windows.borrow().len());
        }
    }

    /// Reposition all notification windows
    fn reposition_all(
        config: &Arc<RwLock<Config>>,
        windows: &Rc<RefCell<HashMap<u32, NotificationWindow>>>,
    ) {
        let config_read = config.read();
        let windows_ref = windows.borrow();

        for (index, (_, window)) in windows_ref.iter().enumerate() {
            window.update_position(&config_read, index);
        }

        debug!("Repositioned {} notifications", windows_ref.len());
    }

    /// Get the GTK application
    pub fn application(&self) -> &Application {
        &self.app
    }
}
