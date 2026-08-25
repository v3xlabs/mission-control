use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{version, HumanDuration, NotificationMode, Stinger};

/// `notifications.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationsDocument {
    #[serde(default = "version::current")]
    pub version: u32,
    #[serde(default)]
    pub mode: NotificationMode,
    #[serde(default = "default_duration")]
    pub default_duration: HumanDuration,
    /// How wide the sidebar window asks to be, in pixels. The compositor has the final say.
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    /// Named clips a tab or a notification can ask for.
    #[serde(default)]
    pub stingers: HashMap<String, Stinger>,
}

fn default_duration() -> HumanDuration {
    HumanDuration(std::time::Duration::from_secs(20))
}

const fn default_sidebar_width() -> u32 {
    480
}

impl Default for NotificationsDocument {
    fn default() -> Self {
        Self {
            version: version::CURRENT,
            mode: NotificationMode::default(),
            default_duration: default_duration(),
            sidebar_width: default_sidebar_width(),
            stingers: HashMap::new(),
        }
    }
}

impl NotificationsDocument {
    pub fn stinger(&self, name: &str) -> Option<&Stinger> {
        self.stingers.get(name)
    }
}
