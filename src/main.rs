#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use parking_lot::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod compositor;
mod config;
mod dbus;
mod dnd;
mod history;
mod ipc;
mod mpris;
mod notification;
mod positioning;
mod rules;
mod ui;

#[cfg(feature = "sound")]
mod sound;

use compositor::CompositorIpc;
use config::ConfigLoader;
use dbus::start_dbus_server_with_history;
use dnd::DndState;
use history::HistoryStore;
use ipc::start_ipc_server_with_ui;
use notification::{ActionEvent, NotificationManager, UiEvent};
use ui::SwaynotiApp;

/// Swaynoti - A modern Wayland notification daemon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,
}

fn setup_logging(debug: bool) {
    let level = if debug { Level::DEBUG } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    setup_logging(args.debug);

    info!("Starting swaynoti v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = if let Some(ref path) = args.config {
        ConfigLoader::load_from_path(&path.into())?
    } else {
        ConfigLoader::load()?
    };

    let config = Arc::new(RwLock::new(config));
    info!("Configuration loaded");

    // Initialize history store
    let history_max = config.read().history.max_entries;
    let history_store = match HistoryStore::new(history_max) {
        Ok(store) => Some(Arc::new(store)),
        Err(e) => {
            tracing::warn!("Failed to initialize history store: {}", e);
            None
        }
    };

    // Create communication channels
    let (ui_sender, ui_receiver) = async_channel::unbounded::<UiEvent>();
    let (action_sender, action_receiver) = async_channel::unbounded::<ActionEvent>();
    let (close_sender, close_receiver) =
        async_channel::unbounded::<(u32, notification::CloseReason)>();

    // Create DND state
    let dnd_state = Arc::new(DndState::new());

    // Create notification manager
    let manager = Arc::new(NotificationManager::new(
        config.clone(),
        ui_sender.clone(),
        close_sender,
    ));

    // Start the tokio runtime for async tasks
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Clone for async tasks
    let dbus_manager = manager.clone();
    let dbus_history = history_store.clone();
    let ipc_manager = manager.clone();
    let ipc_dnd = dnd_state.clone();
    let ipc_config = config.clone();

    // Spawn async tasks
    runtime.spawn(async move {
        // Start D-Bus server with history
        match start_dbus_server_with_history(dbus_manager, close_receiver, dbus_history).await {
            Ok(_connection) => {
                // Keep the connection alive forever
                // The connection must stay in scope for the D-Bus name to remain registered
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                }
            }
            Err(e) => {
                tracing::error!("D-Bus server error: {}", e);
            }
        }
    });

    // Start IPC server with UI sender for notification center commands
    let ipc_ui_sender = ui_sender.clone();
    runtime.spawn(async move {
        let socket_path = ipc_config.read().ipc.socket_path.clone();
        if let Err(e) =
            start_ipc_server_with_ui(ipc_manager, ipc_dnd, socket_path, Some(ipc_ui_sender)).await
        {
            tracing::error!("IPC server error: {}", e);
        }
    });

    // Start DND scheduler
    let dnd_config = config.read().dnd.clone();
    let scheduler_dnd = dnd_state.clone();
    runtime.spawn(async move {
        let scheduler = dnd::DndScheduler::new(dnd_config, scheduler_dnd);
        scheduler.run().await;
    });

    // Handle action events from UI
    let action_manager = manager.clone();
    let action_history = history_store.clone();
    runtime.spawn(async move {
        while let Ok(event) = action_receiver.recv().await {
            match event {
                ActionEvent::ActionInvoked { id, action_key } => {
                    info!("Action '{}' invoked on notification {}", action_key, id);
                    action_manager.invoke_action(id, &action_key).await;
                }
                ActionEvent::Dismissed { id } => {
                    info!("Notification {} dismissed by user", id);
                    if let Some(ref store) = action_history {
                        let _ = store.mark_dismissed(id);
                    }
                    action_manager
                        .close_notification(id, notification::CloseReason::Dismissed)
                        .await;
                }
                ActionEvent::Hovered { id } => {
                    action_manager.set_hovered(id, true);
                }
                ActionEvent::Unhovered { id } => {
                    action_manager.set_hovered(id, false);
                }
                ActionEvent::FocusApp { id, app_name } => {
                    info!("Focusing app '{}' for notification {}", app_name, id);
                    CompositorIpc::focus_window(&app_name);
                    // Also dismiss the notification after focusing
                    action_manager
                        .close_notification(id, notification::CloseReason::Dismissed)
                        .await;
                }
                ActionEvent::InlineReply { id, text } => {
                    info!("Inline reply for notification {}: {}", id, text);
                    // Send the reply as an action with the text
                    action_manager
                        .invoke_action(id, &format!("inline-reply:{}", text))
                        .await;
                    action_manager
                        .close_notification(id, notification::CloseReason::Dismissed)
                        .await;
                }
                ActionEvent::DefaultAction { id } => {
                    info!("Default action triggered for notification {}", id);
                    action_manager.invoke_action(id, "default").await;
                }
            }
        }
    });

    info!("Async services started");

    // Initialize GTK (must be on main thread)
    gtk4::init().expect("Failed to initialize GTK4");

    // Create and run the GTK application
    let app = SwaynotiApp::new(config, action_sender, history_store);

    info!("Starting GTK application");
    app.run(ui_receiver);

    info!("Swaynoti shutting down");
    Ok(())
}
