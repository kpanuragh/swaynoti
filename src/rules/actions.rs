use crate::config::RuleActions;
use crate::notification::{Notification, Urgency};

/// Apply rule actions to a notification
pub fn apply_rule_actions(notification: &mut Notification, actions: &RuleActions) {
    // Override timeout
    if let Some(timeout) = actions.timeout {
        notification.expire_timeout = timeout;
    }

    // Override urgency
    if let Some(ref urgency_str) = actions.urgency {
        notification.hints.urgency = match urgency_str.to_lowercase().as_str() {
            "low" => Urgency::Low,
            "normal" => Urgency::Normal,
            "critical" => Urgency::Critical,
            _ => notification.hints.urgency,
        };
    }

    // Note: Other actions like skip_history, skip_sound, css_class
    // are handled elsewhere in the notification pipeline
}
