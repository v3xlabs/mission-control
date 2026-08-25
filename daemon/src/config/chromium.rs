use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChromiumConfig {
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
    #[serde(default)]
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
            fullscreen: false,
            extra_args: Vec::new(),
        }
    }
}
