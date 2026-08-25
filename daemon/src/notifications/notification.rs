use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::config::NotificationMode;

use super::Level;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct Notification {
    pub notification_id: u64,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    pub level: Level,
    pub mode: NotificationMode,
    /// Seconds from now until it expires, so a page can count down without a shared clock.
    pub expires_in_seconds: u64,
    /// A tab to put on screen instead of a message card. This is what turns a doorbell alert into
    /// the camera feed rather than the word "doorbell".
    #[serde(default)]
    pub tab_id: Option<String>,
    /// A clip played while the alert arrives, by name from `notifications.toml`.
    #[serde(default)]
    pub stinger: Option<String>,
}
