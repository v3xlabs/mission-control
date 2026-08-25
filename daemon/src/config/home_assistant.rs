use serde::{Deserialize, Serialize};

use super::SecretRef;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HomeAssistantConfig {
    pub mqtt_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<SecretRef>,
}
