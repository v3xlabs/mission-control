use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChromiumConfig {
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
    /// Covers the output and hides the tab strip, the omnibox and the profile button. Turning it
    /// off leaves a window the compositor can tile, which is what a layer surface needs to
    /// reserve space beside the page, at the cost of the browser drawing its own chrome.
    #[serde(default = "enabled_by_default")]
    pub fullscreen: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

const fn enabled_by_default() -> bool {
    true
}

impl Default for ChromiumConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            binary_path: None,
            fullscreen: true,
            extra_args: Vec::new(),
        }
    }
}
