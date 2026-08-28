use chrono::{DateTime, Local};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::config::NotificationMode;

use super::Level;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct Notification {
    pub notification_id: u64,
    /// Two pushes carrying one key are one alert said twice. A calendar poll re-derives the same
    /// occurrence every cycle, and this is what stops that becoming a second card.
    #[serde(default)]
    pub key: Option<String>,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    pub level: Level,
    pub mode: NotificationMode,
    /// Seconds from now until it expires, so a page can count down without a shared clock.
    pub expires_in_seconds: u64,
    /// When the thing this alert is about happens. An alert without one renders as a message
    /// rather than as a time.
    #[serde(default)]
    pub starts_at: Option<DateTime<Local>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Local>>,
    #[serde(default)]
    pub location: Option<String>,
    /// A tab to put on screen instead of a message card, so an alert about a camera shows the
    /// stream rather than a sentence describing it.
    #[serde(default)]
    pub tab_id: Option<String>,
    /// A clip played while the alert arrives, by name from `notifications.toml`.
    #[serde(default)]
    pub stinger: Option<String>,
}
