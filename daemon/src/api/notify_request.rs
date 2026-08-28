use chrono::{DateTime, Local};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    config::{HumanDuration, NotificationMode, NotificationsDocument},
    notifications::{Level, Notification},
};

use super::{ApiError, ApiResult};

/// One endpoint everything can reach: a Home Assistant automation, a Stream Deck key, a CI job.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotifyRequest {
    pub title: String,
    pub body: Option<String>,
    pub level: Option<Level>,
    /// Overrides `mode` from `notifications.toml` for this one alert.
    pub mode: Option<NotificationMode>,
    /// A duration such as `20s`. Falls back to the configured default.
    pub duration: Option<String>,
    /// Two calls carrying one key are one alert said twice: the second replaces the first rather
    /// than stacking beside it. An automation that fires on every door event gets this for free.
    pub key: Option<String>,
    /// When the thing this alert is about happens, so the card can show a time and count down.
    pub starts_at: Option<DateTime<Local>>,
    pub ends_at: Option<DateTime<Local>>,
    pub location: Option<String>,
    /// Show this tab rather than a message card, which is what turns a doorbell alert into the
    /// camera feed instead of the word "doorbell".
    pub tab_id: Option<String>,
    /// A clip to cover the change, by name from `notifications.toml`.
    pub stinger: Option<String>,
}

impl NotifyRequest {
    pub fn into_notification(
        self,
        defaults: &NotificationsDocument,
    ) -> ApiResult<(Notification, HumanDuration)> {
        let duration = match self.duration.as_deref() {
            Some(text) => HumanDuration::parse(text).map_err(ApiError::bad_request)?,
            None => defaults.default_duration,
        };

        Ok((
            Notification {
                notification_id: 0,
                key: self.key,
                title: self.title,
                body: self.body,
                level: self.level.unwrap_or_default(),
                mode: self.mode.unwrap_or(defaults.mode),
                expires_in_seconds: duration.seconds(),
                starts_at: self.starts_at,
                ends_at: self.ends_at,
                location: self.location,
                tab_id: self.tab_id,
                stinger: self.stinger,
            },
            duration,
        ))
    }
}
