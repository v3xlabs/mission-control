use serde::{Deserialize, Serialize};

use super::{version, ChromiumConfig, HomeAssistantConfig, HttpConfig, SecretRef};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceDocument {
    #[serde(default = "version::current")]
    pub version: u32,
    pub name: String,
    pub device_id: String,
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_key: Option<SecretRef>,
    #[serde(default)]
    pub chromium: ChromiumConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homeassistant: Option<HomeAssistantConfig>,
}

impl Default for DeviceDocument {
    fn default() -> Self {
        Self {
            version: version::CURRENT,
            name: "Mission Control".to_string(),
            device_id: "missiond".to_string(),
            http: HttpConfig::default(),
            admin_key: None,
            chromium: ChromiumConfig::default(),
            homeassistant: None,
        }
    }
}
