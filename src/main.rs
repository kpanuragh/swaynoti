#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use parking_lot::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod dbus;
mod dnd;
mod ipc;
mod notification;
mod positioning;
mod rules;
mod ui;

#[cfg(feature = "sound")]
mod sound;

use config::ConfigLoader;
use dbus::start_dbus_server;
use dnd::DndState;
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
        ui_sender,
        close_sender,
    ));

    // Start the tokio runtime for async tasks
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Clone for async tasks
    let dbus_manager = manager.clone();
    let ipc_manager = manager.clone();
    let ipc_dnd = dnd_state.clone();
    let ipc_config = config.clone();

    // Spawn async tasks
    runtime.spawn(async move {
        // Start D-Bus server
        match start_dbus_server(dbus_manager, close_receiver).await {
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

    // Start IPC server
    runtime.spawn(async move {
        let socket_path = ipc_config.read().ipc.socket_path.clone();
        if let Err(e) = ipc::start_ipc_server(ipc_manager, ipc_dnd, socket_path).await {
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
    runtime.spawn(async move {
        while let Ok(event) = action_receiver.recv().await {
            match event {
                ActionEvent::ActionInvoked { id, action_key } => {
                    action_manager.invoke_action(id, &action_key).await;
                }
                ActionEvent::Dismissed { id } => {
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
            }
        }
    });

    info!("Async services started");

    // Initialize GTK (must be on main thread)
    gtk4::init().expect("Failed to initialize GTK4");

    // Create and run the GTK application
    let app = SwaynotiApp::new(config, action_sender);

    info!("Starting GTK application");
    app.run(ui_receiver);

    info!("Swaynoti shutting down");
    Ok(())
}
