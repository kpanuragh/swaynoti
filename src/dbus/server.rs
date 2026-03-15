use std::sync::Arc;

use anyhow::{Context, Result};
use async_channel::Receiver;
use tracing::info;
use zbus::connection::Builder;
use zbus::Connection;

use super::interface::NotificationServer;
use crate::history::HistoryStore;
use crate::notification::{ActionEvent, CloseReason, NotificationManager};

/// Start the D-Bus notification server
pub async fn start_dbus_server(
    manager: Arc<NotificationManager>,
    close_receiver: Receiver<(u32, CloseReason)>,
    action_receiver: Receiver<ActionEvent>,
) -> Result<Connection> {
    start_dbus_server_with_history(manager, close_receiver, action_receiver, None).await
}

/// Start the D-Bus notification server with history storage
pub async fn start_dbus_server_with_history(
    manager: Arc<NotificationManager>,
    close_receiver: Receiver<(u32, CloseReason)>,
    action_receiver: Receiver<ActionEvent>,
    history_store: Option<Arc<HistoryStore>>,
) -> Result<Connection> {
    info!("Starting D-Bus notification server...");

    let server = NotificationServer::new(manager);
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

    // Spawn task to handle close events and emit NotificationClosed signals
    let close_conn = connection.clone();
    tokio::spawn(async move {
        let object_server = close_conn.object_server();
        while let Ok((id, reason)) = close_receiver.recv().await {
            info!(
                "Emitting NotificationClosed signal: id={}, reason={:?}",
                id, reason
            );
            // Get interface reference to emit signal
            if let Ok(iface_ref) = object_server
                .interface::<_, NotificationServer>("/org/freedesktop/Notifications")
                .await
            {
                if let Err(e) = NotificationServer::notification_closed(
                    iface_ref.signal_emitter(),
                    id,
                    reason as u32,
                )
                .await
                {
                    tracing::error!("Failed to emit NotificationClosed signal: {}", e);
                }
            }
        }
    });

    // Spawn task to handle action events and emit ActionInvoked signals
    let action_conn = connection.clone();
    tokio::spawn(async move {
        let object_server = action_conn.object_server();
        while let Ok(event) = action_receiver.recv().await {
            match event {
                ActionEvent::ActionInvoked { id, action_key } => {
                    info!("Emitting ActionInvoked signal: id={}, action={}", id, action_key);
                    // Get interface reference to emit signal
                    if let Ok(iface_ref) = object_server
                        .interface::<_, NotificationServer>("/org/freedesktop/Notifications")
                        .await
                    {
                        if let Err(e) = NotificationServer::action_invoked(
                            iface_ref.signal_emitter(),
                            id,
                            action_key,
                        )
                        .await
                        {
                            tracing::error!("Failed to emit ActionInvoked signal: {}", e);
                        }
                    }
                }
                _ => {
                    // Other action events (dismissed, hover, etc.) are handled elsewhere
                }
            }
        }
    });

    Ok(connection)
}
