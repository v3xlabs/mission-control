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
                title: self.title,
                body: self.body,
                level: self.level.unwrap_or_default(),
                mode: self.mode.unwrap_or(defaults.mode),
                expires_in_seconds: duration.seconds(),
                tab_id: self.tab_id,
                stinger: self.stinger,
            },
            duration,
        ))
    }
}
