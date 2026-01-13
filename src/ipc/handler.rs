use std::sync::Arc;

use async_channel::Sender;

use crate::dnd::DndState;
use crate::notification::{NotificationManager, UiEvent};

use super::commands::{IpcCommand, IpcResponse};

/// Handles IPC commands
pub struct IpcHandler {
    manager: Arc<NotificationManager>,
    dnd_state: Arc<DndState>,
    ui_sender: Option<Sender<UiEvent>>,
}

impl IpcHandler {
    pub fn new(manager: Arc<NotificationManager>, dnd_state: Arc<DndState>) -> Self {
        Self {
            manager,
            dnd_state,
            ui_sender: None,
        }
    }

    /// Set the UI sender for notification center commands
    pub fn with_ui_sender(mut self, sender: Sender<UiEvent>) -> Self {
        self.ui_sender = Some(sender);
        self
    }

    /// Handle an IPC command and return a response
    pub async fn handle(&self, command: IpcCommand) -> IpcResponse {
        match command {
            IpcCommand::Dismiss { id } => {
                self.manager
                    .close_notification(id, crate::notification::CloseReason::Dismissed)
                    .await;
                IpcResponse::success()
            }
            IpcCommand::DismissAll => {
                self.manager.dismiss_all().await;
                IpcResponse::success()
            }
            IpcCommand::ToggleDnd => {
                self.dnd_state.toggle();
                IpcResponse::with_data(self.dnd_state.is_enabled())
            }
            IpcCommand::EnableDnd => {
                self.dnd_state.enable();
                IpcResponse::success()
            }
            IpcCommand::DisableDnd => {
                self.dnd_state.disable();
                IpcResponse::success()
            }
            IpcCommand::GetDndStatus => IpcResponse::with_data(self.dnd_state.is_enabled()),
            IpcCommand::ShowHistory | IpcCommand::ShowCenter => {
                if let Some(ref sender) = self.ui_sender {
                    let _ = sender.send(UiEvent::ShowCenter).await;
                }
                IpcResponse::success()
            }
            IpcCommand::HideHistory | IpcCommand::HideCenter => {
                if let Some(ref sender) = self.ui_sender {
                    let _ = sender.send(UiEvent::HideCenter).await;
                }
                IpcResponse::success()
            }
            IpcCommand::ToggleCenter => {
                if let Some(ref sender) = self.ui_sender {
                    let _ = sender.send(UiEvent::ToggleCenter).await;
                }
                IpcResponse::success()
            }
            IpcCommand::GetCount => IpcResponse::with_data(self.manager.count()),
            IpcCommand::ReloadConfig => {
                // TODO: Implement config reload
                IpcResponse::success()
            }
            IpcCommand::GetNotifications => {
                let notifications = self.manager.get_visible_notifications();
                let summaries: Vec<_> = notifications
                    .iter()
                    .map(|n| {
                        serde_json::json!({
                            "id": n.id,
                            "app": n.app_name,
                            "summary": n.summary,
                            "urgency": n.hints.urgency.to_string(),
                        })
                    })
                    .collect();
                IpcResponse::with_data(summaries)
            }
            IpcCommand::InvokeAction { id, action } => {
                self.manager.invoke_action(id, &action).await;
                IpcResponse::success()
            }
        }
    }
}
