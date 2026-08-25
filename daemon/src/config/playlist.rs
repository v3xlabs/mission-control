use serde::{Deserialize, Serialize};

use super::HumanDuration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Playlist {
    pub playlist_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub interval: HumanDuration,
    /// How long a tab chosen by hand holds the display before rotation resumes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold: Option<HumanDuration>,
    #[serde(default)]
    pub is_default: bool,
    /// The list order is the play order.
    #[serde(default)]
    pub tabs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled_tabs: Vec<String>,
}

impl Playlist {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.playlist_id)
    }

    pub fn enabled_tabs(&self) -> impl Iterator<Item = &String> {
        self.tabs
            .iter()
            .filter(|tab_id| !self.disabled_tabs.contains(tab_id))
    }
}
