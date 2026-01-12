use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Control utility for swaynoti daemon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the IPC socket
    #[arg(short, long)]
    socket: Option<PathBuf>,

    /// Output JSON format
    #[arg(short, long)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Dismiss a notification by ID
    Dismiss {
        /// Notification ID
        id: u32,
    },
    /// Dismiss all notifications
    DismissAll,
    /// Toggle Do Not Disturb mode
    ToggleDnd,
    /// Enable Do Not Disturb mode
    EnableDnd,
    /// Disable Do Not Disturb mode
    DisableDnd,
    /// Get Do Not Disturb status
    DndStatus,
    /// Show notification history panel
    ShowHistory,
    /// Hide notification history panel
    HideHistory,
    /// Get count of active notifications
    Count,
    /// Reload configuration
    Reload,
    /// List active notifications
    List,
    /// Invoke an action on a notification
    Action {
        /// Notification ID
        id: u32,
        /// Action key
        action: String,
    },
}

#[derive(Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum IpcCommand {
    Dismiss { id: u32 },
    DismissAll,
    ToggleDnd,
    EnableDnd,
    DisableDnd,
    GetDndStatus,
    ShowHistory,
    HideHistory,
    GetCount,
    ReloadConfig,
    GetNotifications,
    InvokeAction { id: u32, action: String },
}

#[derive(Deserialize)]
struct IpcResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

fn get_socket_path(custom: Option<PathBuf>) -> PathBuf {
    custom.unwrap_or_else(|| {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(runtime_dir).join("swaynoti.sock")
    })
}

fn send_command(socket_path: &PathBuf, command: IpcCommand) -> Result<IpcResponse> {
    let mut stream = UnixStream::connect(socket_path)
        .with_context(|| format!("Failed to connect to socket: {:?}", socket_path))?;

    let json = serde_json::to_string(&command)? + "\n";
    stream.write_all(json.as_bytes())?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    let response: IpcResponse =
        serde_json::from_str(&response).context("Failed to parse response")?;

    Ok(response)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let socket_path = get_socket_path(args.socket);

    let command = match args.command {
        Command::Dismiss { id } => IpcCommand::Dismiss { id },
        Command::DismissAll => IpcCommand::DismissAll,
        Command::ToggleDnd => IpcCommand::ToggleDnd,
        Command::EnableDnd => IpcCommand::EnableDnd,
        Command::DisableDnd => IpcCommand::DisableDnd,
        Command::DndStatus => IpcCommand::GetDndStatus,
        Command::ShowHistory => IpcCommand::ShowHistory,
        Command::HideHistory => IpcCommand::HideHistory,
        Command::Count => IpcCommand::GetCount,
        Command::Reload => IpcCommand::ReloadConfig,
        Command::List => IpcCommand::GetNotifications,
        Command::Action { id, action } => IpcCommand::InvokeAction { id, action },
    };

    let response = send_command(&socket_path, command)?;

    if args.json {
        // JSON output
        let output = serde_json::json!({
            "success": response.success,
            "data": response.data,
            "error": response.error,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable output
        if response.success {
            if let Some(data) = response.data {
                match data {
                    serde_json::Value::Bool(b) => {
                        println!("{}", if b { "enabled" } else { "disabled" });
                    }
                    serde_json::Value::Number(n) => {
                        println!("{}", n);
                    }
                    serde_json::Value::Array(arr) => {
                        for item in arr {
                            if let Some(obj) = item.as_object() {
                                let id = obj.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                                let app = obj.get("app").and_then(|v| v.as_str()).unwrap_or("");
                                let summary =
                                    obj.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                                let urgency =
                                    obj.get("urgency").and_then(|v| v.as_str()).unwrap_or("");
                                println!("[{}] {} - {} ({})", id, app, summary, urgency);
                            }
                        }
                    }
                    _ => {
                        println!("{}", data);
                    }
                }
            } else {
                println!("OK");
            }
        } else {
            eprintln!(
                "Error: {}",
                response.error.unwrap_or_else(|| "Unknown error".into())
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
