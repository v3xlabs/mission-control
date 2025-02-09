use bytes::Bytes;
use rumqttc::{Client, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HassEntity {
    pub name: String,
    pub icon: String,
    pub unique_id: String,
    pub device_class: String,
    pub device: HassDevice,
    pub command_topic: String,
    pub state_topic: String,

    // Availability
    pub availability_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_available: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_not_available: Option<String>,

    // Switch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_off: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_off: Option<String>,

    // Number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f32>,

    // Select
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,

    #[serde(skip)]
    pub config_topic: String,

    #[serde(flatten)]
    pub extra: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HassDevice {
    pub identifiers: Vec<String>,
    pub name: String,
    pub configuration_url: String,
    pub serial_number: String,
}

impl HassEntity {
    pub fn new_backlight(name: String, unique_id: String, availability_topic: String) -> Self {
        Self {
            name: "Backlight".to_string(),
            icon: "mdi:monitor".to_string(),
            unique_id: format!("{unique_id}_backlight", unique_id = unique_id),
            device_class: "switch".to_string(),
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
            options: None,
            extra: None,
        }
    }

    pub fn new_brightness(name: String, unique_id: String, availability_topic: String) -> Self {
        Self {
            name: "Brightness".to_string(),
            icon: "mdi:brightness-7".to_string(),
            unique_id: format!("{unique_id}_brightness", unique_id = unique_id),
            device_class: "number".to_string(),
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
            options: None,
            extra: None,
        }
    }

    pub fn new_playlist(
        name: String,
        unique_id: String,
        availability_topic: String,
        options: Option<Vec<String>>,
    ) -> Self {
        Self {
            name: "Playlist".to_string(),
            icon: "mdi:playlist-play".to_string(),
            unique_id: format!("{unique_id}_playlist", unique_id = unique_id),
            device_class: "select".to_string(),
            device: HassDevice {
                identifiers: vec![unique_id.clone()],
                name,
                configuration_url: "https://v3x.fyi/s1".to_string(),
                serial_number: unique_id.clone(),
            },
            state_topic: format!(
                "homeassistant/select/{unique_id}_playlist/state",
                unique_id = unique_id
            ),
            command_topic: format!(
                "homeassistant/select/{unique_id}_playlist/set",
                unique_id = unique_id
            ),
            config_topic: format!(
                "homeassistant/select/{unique_id}_playlist/config",
                unique_id = unique_id
            ),
            availability_topic,
            state_on: None,
            state_off: None,
            payload_on: None,
            payload_off: None,
            payload_available: None,
            payload_not_available: None,
            min: None,
            max: None,
            step: None,
            options,
            extra: None,
        }
    }

    pub fn publish_config(&self, client: &Client) {
        let payload_str: String = serde_json::to_string(&self).unwrap();
        client
            .publish(&self.config_topic, QoS::AtLeastOnce, true, payload_str)
            .unwrap();
    }

    pub fn subscribe(&self, client: &Client) {
        client
            .subscribe(&self.command_topic, QoS::AtMostOnce)
            .unwrap();
    }

    pub fn handle_command(&self, client: &Client, command: &Bytes) {
        println!("Command received: {:?}", command);
        let command: &[u8] = command.as_ref();

        if self.device_class == "switch" {
            if command.eq(b"ON") {
                self.update_state(client, "ON");
            } else if command.eq(b"OFF") {
                self.update_state(client, "OFF");
            }
        } else if self.device_class == "number" {
            let command: &str = std::str::from_utf8(command).unwrap();
            self.update_state(client, command);
        } else if self.device_class == "select" {
            let command: &str = std::str::from_utf8(command).unwrap();
            self.update_state(client, command);
        }
    }

    pub fn update_state(&self, client: &Client, state: &str) {
        client
            .publish(&self.state_topic, QoS::AtLeastOnce, true, state)
            .unwrap();
    }
}
