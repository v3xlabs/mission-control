use anyhow::Result;
use figment::{providers::{Format, Toml}, Figment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub homeassistant: HomeAssistantConfig,
    pub device: DeviceConfig,
    pub display: DisplayConfig,
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

pub fn load_config() -> Result<Config> {
    let figment = Figment::new().merge(Toml::file("config.toml"));
    let config = figment.extract::<Config>()?;
    Ok(config)
}
