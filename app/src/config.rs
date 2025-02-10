use std::collections::HashMap;

use anyhow::Result;
use figment::{providers::{Format, Toml}, Figment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub homeassistant: HomeAssistantConfig,
    pub device: DeviceConfig,
    pub display: DisplayConfig,
    pub chromium: Option<ChromiumConfig>,
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantConfig {
    pub mqtt_url: String,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceConfig {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct DisplayConfig {
    pub sleep_time: Option<u32>,
    pub xrandr: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChromiumConfig {
    pub enabled: bool,
    pub binary_path: Option<String>,
    pub theme: Option<String>,
    pub tabs: Option<HashMap<String, ChromiumTabConfig>>,
    pub playlists: Option<HashMap<String, ChromiumPlaylistConfig>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChromiumTabConfig {
    pub url: String,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChromiumPlaylistConfig {
    pub tabs: Vec<String>,
    pub interval: u32,
}

pub fn load_config() -> Result<Config> {
    let figment = Figment::new().merge(Toml::file("config.toml"));
    let config = figment.extract::<Config>()?;
    Ok(config)
}
