use regex::Regex;
use tracing::debug;

use crate::config::{AppRule, RuleCriteria};
use crate::notification::Notification;

/// Matches notifications against rules
pub struct RuleMatcher;

impl RuleMatcher {
    /// Find the first matching rule for a notification
    pub fn find_matching_rule<'a>(
        notification: &Notification,
        rules: &'a [AppRule],
    ) -> Option<&'a AppRule> {
        for rule in rules {
            if Self::matches(&rule.criteria, notification) {
                debug!(
                    "Notification {} matched rule for app '{:?}'",
                    notification.id, rule.criteria.app_name
                );
                return Some(rule);
            }
        }
        None
    }

    /// Check if a notification matches the given criteria
    fn matches(criteria: &RuleCriteria, notification: &Notification) -> bool {
        // Check app_name
        if let Some(ref pattern) = criteria.app_name {
            if !Self::matches_pattern(pattern, &notification.app_name) {
                return false;
            }
        }

        // Check summary
        if let Some(ref pattern) = criteria.summary {
            if !Self::matches_pattern(pattern, &notification.summary) {
                return false;
            }
        }

        // Check body
        if let Some(ref pattern) = criteria.body {
            if !Self::matches_pattern(pattern, &notification.body) {
                return false;
            }
        }

        // Check urgency
        if let Some(ref urgency_str) = criteria.urgency {
            let urgency_matches = match urgency_str.to_lowercase().as_str() {
                "low" => notification.hints.urgency == crate::notification::Urgency::Low,
                "normal" => notification.hints.urgency == crate::notification::Urgency::Normal,
                "critical" => notification.hints.urgency == crate::notification::Urgency::Critical,
                _ => true,
            };
            if !urgency_matches {
                return false;
            }
        }

        // Check category
        if let Some(ref category) = criteria.category {
            if let Some(ref notif_category) = notification.hints.category {
                if !Self::matches_pattern(category, notif_category) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Check if a value matches a pattern (supports regex)
    fn matches_pattern(pattern: &str, value: &str) -> bool {
        // Try as regex first
        if let Ok(regex) = Regex::new(pattern) {
            regex.is_match(value)
        } else {
            // Fall back to exact match
            pattern == value
        }
    }
}
