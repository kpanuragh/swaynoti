use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_channel::Sender;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

use crate::dnd::DndState;
use crate::notification::{NotificationManager, UiEvent};

use super::commands::IpcCommand;
use super::handler::IpcHandler;

/// Get the default socket path
pub fn default_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("swaynoti.sock")
}

/// Start the IPC server
pub async fn start_ipc_server(
    manager: Arc<NotificationManager>,
    dnd_state: Arc<DndState>,
    socket_path: Option<PathBuf>,
) -> Result<()> {
    start_ipc_server_with_ui(manager, dnd_state, socket_path, None).await
}

/// Start the IPC server with UI sender for notification center
pub async fn start_ipc_server_with_ui(
    manager: Arc<NotificationManager>,
    dnd_state: Arc<DndState>,
    socket_path: Option<PathBuf>,
    ui_sender: Option<Sender<UiEvent>>,
) -> Result<()> {
    let path = socket_path.unwrap_or_else(default_socket_path);

    // Remove existing socket if present
    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    let listener = UnixListener::bind(&path)?;
    info!("IPC server listening on {:?}", path);

    let handler = IpcHandler::new(manager, dnd_state);
    let handler = if let Some(sender) = ui_sender {
        handler.with_ui_sender(sender)
    } else {
        handler
    };
    let handler = Arc::new(handler);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let handler = handler.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, handler).await {
                        error!("Error handling IPC client: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting IPC connection: {}", e);
            }
        }
    }
}

async fn handle_client(stream: UnixStream, handler: Arc<IpcHandler>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        debug!("Received IPC command: {}", line.trim());

        let response = match serde_json::from_str::<IpcCommand>(&line) {
            Ok(command) => handler.handle(command).await,
            Err(e) => {
                warn!("Invalid IPC command: {}", e);
                super::commands::IpcResponse::error(format!("Invalid command: {}", e))
            }
        };

        let response_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(response_json.as_bytes()).await?;

        line.clear();
    }

    Ok(())
}
