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
    pub state_on: String,
    pub state_off: String,
    pub payload_on: String,
    pub payload_off: String,
    pub command_topic: String,
    pub availability_topic: String,
    pub payload_available: String,
    pub payload_not_available: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HassDevice {
    pub identifiers: Vec<String>,
    pub name: String,
}

impl HassDeviceDiscoveryPayload {
    pub fn new(
        name: String,
        unique_id: String,
        state_topic: String,
        command_topic: String,
        availability_topic: String,
    ) -> Self {
        Self {
            name: name.clone(),
            icon: "mdi:monitor".to_string(),
            unique_id: unique_id.clone(),
            // device_class: "switch".to_string(),
            device: HassDevice {
                identifiers: vec![unique_id],
                name,
            },
            state_topic,
            command_topic,
            availability_topic,
            state_on: "ON".to_string(),
            state_off: "OFF".to_string(),
            payload_on: "ON".to_string(),
            payload_off: "OFF".to_string(),
            payload_available: "online".to_string(),
            payload_not_available: "offline".to_string(),
        }
    }
}
