use serde::{Deserialize, Serialize};

/// The player that draws camera tabs.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MpvConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

impl MpvConfig {
    pub fn binary(&self) -> String {
        self.binary_path
            .clone()
            .or_else(|| std::env::var("MPV_BINARY").ok())
            .unwrap_or_else(|| "mpv".to_string())
    }
}
