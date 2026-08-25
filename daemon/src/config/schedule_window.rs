use serde::{Deserialize, Serialize};

use super::Weekday;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduleWindow {
    pub days: Vec<Weekday>,
    /// `HH:MM`, local time.
    pub from: String,
    pub to: String,
}
