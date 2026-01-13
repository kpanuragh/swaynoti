use std::sync::Arc;

use anyhow::{Context, Result};
use async_channel::Receiver;
use tracing::info;
use zbus::connection::Builder;
use zbus::Connection;

use super::interface::NotificationServer;
use crate::history::HistoryStore;
use crate::notification::{CloseReason, NotificationManager};

/// Start the D-Bus notification server
pub async fn start_dbus_server(
    manager: Arc<NotificationManager>,
    close_receiver: Receiver<(u32, CloseReason)>,
) -> Result<Connection> {
    start_dbus_server_with_history(manager, close_receiver, None).await
}

/// Start the D-Bus notification server with history storage
pub async fn start_dbus_server_with_history(
    manager: Arc<NotificationManager>,
    close_receiver: Receiver<(u32, CloseReason)>,
    history_store: Option<Arc<HistoryStore>>,
) -> Result<Connection> {
    info!("Starting D-Bus notification server...");

    let server = NotificationServer::new(manager, close_receiver);
    let server = if let Some(store) = history_store {
        server.with_history(store)
    } else {
        server
    };

    let connection = Builder::session()
        .context("Failed to connect to session bus")?
        .name("org.freedesktop.Notifications")
        .context("Failed to request notification service name")?
        .serve_at("/org/freedesktop/Notifications", server)
        .context("Failed to serve notification interface")?
        .build()
        .await
        .context("Failed to build D-Bus connection")?;

    info!("D-Bus server started successfully");
    info!("Registered as org.freedesktop.Notifications");

    Ok(connection)
}
