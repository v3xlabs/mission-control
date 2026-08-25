use rumqttc::{AsyncClient, QoS};
use serde::Serialize;
use tracing::warn;

use crate::config::DeviceDocument;

use super::HassDevice;

#[derive(Debug, Clone, Serialize)]
pub struct HassEntity {
    pub name: String,
    pub icon: String,
    pub unique_id: String,
    pub device_class: String,
    pub device: HassDevice,
    pub command_topic: String,
    pub state_topic: String,
    pub availability_topic: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,

    #[serde(skip)]
    pub config_topic: String,
}

impl HassEntity {
    fn build(
        device: &DeviceDocument,
        availability_topic: &str,
        kind: &str,
        slug: &str,
        name: &str,
        icon: &str,
    ) -> Self {
        let device_id = &device.device_id;

        Self {
            name: name.to_string(),
            icon: icon.to_string(),
            unique_id: format!("{device_id}_{slug}"),
            device_class: kind.to_string(),
            device: HassDevice {
                identifiers: vec![device_id.clone()],
                name: device.name.clone(),
                serial_number: device_id.clone(),
            },
            state_topic: format!("homeassistant/{kind}/{device_id}_{slug}/state"),
            command_topic: format!("homeassistant/{kind}/{device_id}_{slug}/set"),
            config_topic: format!("homeassistant/{kind}/{device_id}_{slug}/config"),
            availability_topic: availability_topic.to_string(),
            min: None,
            max: None,
            step: None,
            options: None,
        }
    }

    pub fn new_backlight(device: &DeviceDocument, availability_topic: &str) -> Self {
        Self::build(
            device,
            availability_topic,
            "switch",
            "backlight",
            "Backlight",
            "mdi:monitor",
        )
    }

    pub fn new_brightness(device: &DeviceDocument, availability_topic: &str) -> Self {
        Self {
            min: Some(0.0),
            max: Some(1.0),
            step: Some(0.01),
            ..Self::build(
                device,
                availability_topic,
                "number",
                "brightness",
                "Brightness",
                "mdi:brightness-7",
            )
        }
    }

    pub fn new_playlist(device: &DeviceDocument, availability_topic: &str) -> Self {
        Self {
            options: Some(Vec::new()),
            ..Self::build(
                device,
                availability_topic,
                "select",
                "playlist",
                "Playlist",
                "mdi:playlist-play",
            )
        }
    }

    pub fn new_tab(device: &DeviceDocument, availability_topic: &str) -> Self {
        Self {
            options: Some(Vec::new()),
            ..Self::build(
                device,
                availability_topic,
                "select",
                "tab",
                "Tab",
                "mdi:tab",
            )
        }
    }

    pub fn new_url(device: &DeviceDocument, availability_topic: &str) -> Self {
        Self::build(
            device,
            availability_topic,
            "sensor",
            "url",
            "URL",
            "mdi:link",
        )
    }

    pub async fn publish_config(&self, client: &AsyncClient) {
        let payload = match serde_json::to_vec(self) {
            Ok(payload) => payload,
            Err(error) => {
                warn!("cannot serialise {} config: {error}", self.unique_id);
                return;
            }
        };

        if let Err(error) = client
            .publish(&self.config_topic, QoS::AtLeastOnce, true, payload)
            .await
        {
            warn!("cannot publish {} config: {error}", self.unique_id);
        }
    }

    pub async fn update_state(&self, client: &AsyncClient, state: &str) {
        if let Err(error) = client
            .publish(&self.state_topic, QoS::AtLeastOnce, true, state)
            .await
        {
            warn!("cannot publish {} state: {error}", self.unique_id);
        }
    }

    pub async fn subscribe(&self, client: &AsyncClient) {
        if let Err(error) = client.subscribe(&self.command_topic, QoS::AtLeastOnce).await {
            warn!("cannot subscribe to {}: {error}", self.command_topic);
        }
    }
}
