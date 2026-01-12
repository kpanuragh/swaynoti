use chrono::{Datelike, Local, NaiveTime, Weekday as ChronoWeekday};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

use crate::config::{DndConfig, Weekday};

use super::DndState;

/// Manages scheduled Do Not Disturb
pub struct DndScheduler {
    config: DndConfig,
    state: Arc<DndState>,
}

impl DndScheduler {
    pub fn new(config: DndConfig, state: Arc<DndState>) -> Self {
        Self { config, state }
    }

    /// Start the scheduler (runs in background)
    pub async fn run(self) {
        if self.config.schedule_start.is_none() || self.config.schedule_end.is_none() {
            debug!("DND schedule not configured, scheduler not starting");
            return;
        }

        let start_time = match self.parse_time(&self.config.schedule_start) {
            Some(t) => t,
            None => {
                debug!("Invalid DND schedule start time");
                return;
            }
        };

        let end_time = match self.parse_time(&self.config.schedule_end) {
            Some(t) => t,
            None => {
                debug!("Invalid DND schedule end time");
                return;
            }
        };

        info!(
            "DND scheduler started: {:?} - {:?} on {:?}",
            start_time, end_time, self.config.schedule_days
        );

        let mut check_interval = interval(Duration::from_secs(60));

        loop {
            check_interval.tick().await;

            let now = Local::now();
            let current_time = now.time();
            let current_weekday = now.weekday();

            // Check if today is a scheduled day
            let is_scheduled_day = self.config.schedule_days.is_empty()
                || self
                    .config
                    .schedule_days
                    .iter()
                    .any(|d| Self::weekday_matches(d, current_weekday));

            if !is_scheduled_day {
                self.state.disable_scheduled();
                continue;
            }

            // Check if we're in the DND time range
            let in_range = if start_time <= end_time {
                // Same day range (e.g., 09:00 - 17:00)
                current_time >= start_time && current_time < end_time
            } else {
                // Overnight range (e.g., 22:00 - 08:00)
                current_time >= start_time || current_time < end_time
            };

            if in_range {
                self.state.enable_scheduled();
            } else {
                self.state.disable_scheduled();
            }
        }
    }

    fn parse_time(&self, time_str: &Option<String>) -> Option<NaiveTime> {
        time_str
            .as_ref()
            .and_then(|s| NaiveTime::parse_from_str(s, "%H:%M").ok())
    }

    fn weekday_matches(config_day: &Weekday, chrono_day: ChronoWeekday) -> bool {
        matches!(
            (config_day, chrono_day),
            (Weekday::Monday, ChronoWeekday::Mon)
                | (Weekday::Tuesday, ChronoWeekday::Tue)
                | (Weekday::Wednesday, ChronoWeekday::Wed)
                | (Weekday::Thursday, ChronoWeekday::Thu)
                | (Weekday::Friday, ChronoWeekday::Fri)
                | (Weekday::Saturday, ChronoWeekday::Sat)
                | (Weekday::Sunday, ChronoWeekday::Sun)
        )
    }
}
