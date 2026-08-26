use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::config::{Source, Tab};

/// A tab created through the API is always a page. A camera stream url carries a credential, so
/// it is declared in the config directory rather than posted in and written back to disk.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpsertTabRequest {
    pub name: Option<String>,
    pub url: String,
    pub persist: Option<bool>,
    pub scale: Option<f64>,
    /// A stinger played while this tab loads, by name from `notifications.toml`.
    pub stinger: Option<String>,
}

impl UpsertTabRequest {
    pub fn into_tab(self, tab_id: String) -> Tab {
        Tab {
            tab_id,
            name: self.name,
            source: Source::Url(self.url),
            persist: self.persist.unwrap_or(true),
            scale: self.scale,
            stinger: self.stinger,
        }
    }
}
