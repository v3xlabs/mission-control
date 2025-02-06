use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HassDeviceDiscoveryPayload {
    pub name: String,
    pub icon: String,
    pub unique_id: String,
    // pub device_class: String,
    pub device: HassDevice,
    pub state_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_off: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_off: Option<String>,
    pub command_topic: String,
    pub availability_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_available: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_not_available: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f32>,

    #[serde(skip)]
    pub config_topic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HassDevice {
    pub identifiers: Vec<String>,
    pub name: String,
    pub configuration_url: String,
    pub serial_number: String,
}

impl HassDeviceDiscoveryPayload {
    pub fn new_backlight(name: String, unique_id: String, availability_topic: String) -> Self {
        Self {
            name: format!("{name} Backlight", name = name),
            icon: "mdi:monitor".to_string(),
            unique_id: format!("{unique_id}_backlight", unique_id = unique_id),
            // device_class: "switch".to_string(),
            device: HassDevice {
                identifiers: vec![unique_id.clone()],
                name,
                configuration_url: "https://v3x.fyi/s1".to_string(),
                serial_number: unique_id.clone(),
            },
            state_topic: format!(
                "homeassistant/switch/{unique_id}_backlight/state",
                unique_id = unique_id
            ),
            command_topic: format!(
                "homeassistant/switch/{unique_id}_backlight/set",
                unique_id = unique_id
            ),
            config_topic: format!(
                "homeassistant/switch/{unique_id}_backlight/config",
                unique_id = unique_id
            ),
            availability_topic,
            state_on: Some("ON".to_string()),
            state_off: Some("OFF".to_string()),
            payload_on: Some("ON".to_string()),
            payload_off: Some("OFF".to_string()),
            payload_available: Some("online".to_string()),
            payload_not_available: Some("offline".to_string()),
            min: None,
            max: None,
            step: None,
        }
    }

    pub fn new_brightness(name: String, unique_id: String, availability_topic: String) -> Self {
        Self {
            name: format!("{name} Brightness", name = name),
            icon: "mdi:brightness-7".to_string(),
            unique_id: format!("{unique_id}_brightness", unique_id = unique_id),
            device: HassDevice {
                identifiers: vec![unique_id.clone()],
                name,
                configuration_url: "https://v3x.fyi/s1".to_string(),
                serial_number: unique_id.clone(),
            },
            state_topic: format!(
                "homeassistant/number/{unique_id}_brightness/state",
                unique_id = unique_id
            ),
            command_topic: format!(
                "homeassistant/number/{unique_id}_brightness/set",
                unique_id = unique_id
            ),
            config_topic: format!(
                "homeassistant/number/{unique_id}_brightness/config",
                unique_id = unique_id
            ),
            availability_topic,
            state_on: None,
            state_off: None,
            payload_on: None,
            payload_off: None,
            payload_available: None,
            payload_not_available: None,
            min: Some(0.0),
            max: Some(1.0),
            step: Some(0.01),
        }
    }
}
