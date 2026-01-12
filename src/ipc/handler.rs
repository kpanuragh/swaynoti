use std::sync::Arc;

use crate::notification::NotificationManager;
use crate::dnd::DndState;

use super::commands::{IpcCommand, IpcResponse};

/// Handles IPC commands
pub struct IpcHandler {
    manager: Arc<NotificationManager>,
    dnd_state: Arc<DndState>,
}

impl IpcHandler {
    pub fn new(manager: Arc<NotificationManager>, dnd_state: Arc<DndState>) -> Self {
        Self { manager, dnd_state }
    }

    /// Handle an IPC command and return a response
    pub async fn handle(&self, command: IpcCommand) -> IpcResponse {
        match command {
            IpcCommand::Dismiss { id } => {
                self.manager.close_notification(id, crate::notification::CloseReason::Dismissed).await;
                IpcResponse::success()
            }
            IpcCommand::DismissAll => {
                // TODO: Implement dismiss all
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
            IpcCommand::GetDndStatus => {
                IpcResponse::with_data(self.dnd_state.is_enabled())
            }
            IpcCommand::ShowHistory => {
                // TODO: Implement history panel
                IpcResponse::success()
            }
            IpcCommand::HideHistory => {
                // TODO: Implement history panel
                IpcResponse::success()
            }
            IpcCommand::GetCount => {
                IpcResponse::with_data(self.manager.count())
            }
            IpcCommand::ReloadConfig => {
                // TODO: Implement config reload
                IpcResponse::success()
            }
            IpcCommand::GetNotifications => {
                let notifications = self.manager.get_visible_notifications();
                let summaries: Vec<_> = notifications.iter()
                    .map(|n| serde_json::json!({
                        "id": n.id,
                        "app": n.app_name,
                        "summary": n.summary,
                        "urgency": n.hints.urgency.to_string(),
                    }))
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
