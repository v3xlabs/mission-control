use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tab {
    pub tab_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub url: String,
    /// Whether the page stays loaded when the playlist moves on.
    #[serde(default = "persist_by_default")]
    pub persist: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

const fn persist_by_default() -> bool {
    true
}

impl Tab {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.tab_id)
    }
}
