use std::time::Duration;

use reqwest::Url;
use rumqttc::{Client, Connection, Event, LastWill, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;

pub struct HassManager {
    // connection: Connection,
    pub mqtt_client: Client,
    pub availability_topic: String,
}

impl HassManager {
    pub async fn new(config: &Config) -> (Self, Connection) {
        let mqtt_url = config.homeassistant.mqtt_url.parse::<Url>().unwrap();
        let mqtt_port = mqtt_url.port().unwrap_or(1883);

        let mqtt_url = mqtt_url.host_str().unwrap();

        println!("MQTT URL: {}", mqtt_url);

        let availability_topic = format!("homeassistant/device/{}/availability", config.device.id);
        let availability_lastwill = "offline";

        let mut mqttoptions = MqttOptions::new("mission-control-1", mqtt_url, mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(15));
        mqttoptions.set_last_will(LastWill::new(
            &availability_topic,
            availability_lastwill,
            QoS::AtMostOnce,
            true,
        ));

        if let Some(username) = &config.homeassistant.mqtt_username {
            if let Some(password) = &config.homeassistant.mqtt_password {
                println!(
                    "Setting credentials for MQTT connection {} {}",
                    username, password
                );
                mqttoptions.set_credentials(username, password);
            }
        }

        let (mut client, mut connection) = Client::new(mqttoptions, 10);

        (
            Self {
                mqtt_client: client,
                // connection,
                availability_topic,
            },
            connection,
        )
    }

    pub async fn init(&self) {
        self.mqtt_client
            .publish(&self.availability_topic, QoS::AtLeastOnce, true, "online")
            .unwrap();
    }

    pub fn run(&self, connection: &mut Connection) {
        let mut error_count = 0;

        for (i, notification) in connection.iter().enumerate() {
            println!("Notification: {:?}", notification);

            match notification {
                Ok(payload) => {
                    match payload {
                        Event::Incoming(event) => {
                            match event {
                                Packet::Publish(publish) => {
                                    println!("Publish: {:?}", &publish);

                                    // if publish.topic.eq(&self.discovery_payload_arc.command_topic) {
                                    //     println!("Command received: {:?}", &publish.payload);

                                    //     if publish.payload.eq("ON") {
                                    //         println!("Turning on display");
                                    //         state.hass.mqtt_client
                                    //             .publish(
                                    //                 &discovery_payload_arc.state_topic,
                                    //                 QoS::AtLeastOnce,
                                    //                 true,
                                    //                 "ON",
                                    //             )
                                    //             .unwrap();

                                    //         if let Some(xrandr) = &config.display.xrandr {
                                    //             let xrandr_command = format!(
                                    //                 "xset dpms force on && xset s off && xset -dpms",
                                    //             );
                                    //             let xrandr_result = std::process::Command::new("sh")
                                    //                 .arg("-c")
                                    //                 .arg(xrandr_command)
                                    //                 .output()
                                    //                 .expect("Failed to execute xrandr command");
                                    //             println!("xrandr result: {:?}", xrandr_result);
                                    //         }
                                    //     } else if publish.payload.eq("OFF") {
                                    //         println!("Turning off display");
                                    //         state.hass.mqtt_client
                                    //             .publish(
                                    //                 &discovery_payload_arc.state_topic,
                                    //                 QoS::AtLeastOnce,
                                    //                 true,
                                    //                 "OFF",
                                    //             )
                                    //             .unwrap();

                                    //         if let Some(xrandr) = &config.display.xrandr {
                                    //             let xrandr_command = format!(
                                    //             "xset s off && xset +dpms && xset dpms 600 600 600 && xset dpms force off",
                                    //         );
                                    //             let xrandr_result = std::process::Command::new("sh")
                                    //                 .arg("-c")
                                    //                 .arg(xrandr_command)
                                    //                 .output()
                                    //                 .expect("Failed to execute xrandr command");
                                    //             println!("xrandr result: {:?}", xrandr_result);
                                    //         }
                                    //     }
                                    // }

                                    // if publish
                                    //     .topic
                                    //     .eq(&discovery_payload_brightness_arc.command_topic)
                                    // {
                                    //     println!("Command received: {:?}", &publish.payload);

                                    //     // Convert bytes to string and parse as f32
                                    //     let brightness_str = String::from_utf8_lossy(&publish.payload);
                                    //     let brightness_value: f32 = brightness_str.parse().unwrap();
                                    //     state.hass.mqtt_client
                                    //         .publish(
                                    //             &discovery_payload_brightness_arc.state_topic,
                                    //             QoS::AtLeastOnce,
                                    //             true,
                                    //             brightness_value.to_string().as_str(),
                                    //         )
                                    //         .unwrap();

                                    //     if let Some(xrandr) = &config.display.xrandr {
                                    //         let xrandr_command = format!(
                                    //             "xrandr --output {} --brightness {}",
                                    //             xrandr, brightness_value
                                    //         );
                                    //         println!("xrandr command: {}", xrandr_command);
                                    //         let xrandr_result = std::process::Command::new("sh")
                                    //             .arg("-c")
                                    //             .arg(xrandr_command)
                                    //             .output()
                                    //             .expect("Failed to execute xrandr command");
                                    //         println!("xrandr result: {:?}", xrandr_result);
                                    //     }
                                    // }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    error_count += 1;
                    println!("Error: {:?}", e);

                    // if error_count > 10 {
                    // println!("Too many errors, exiting");
                    // break;
                    // }
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HassDeviceDiscoveryPayload {
    pub name: String,
    pub icon: String,
    pub unique_id: String,
    // pub device_class: String,
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
            options: None,
            extra: None,
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
            options: None,
            extra: None,
        }
    }

    // pub fn new_playlist(name: String, unique_id: String, availability_topic: String) -> Self {
    //     Self {
    //         name: format!("{name} Playlist", name = name),
    //         icon: "mdi:playlist-play".to_string(),
    //         unique_id: format!("{unique_id}_playlist", unique_id = unique_id),

    //     }
    // }
}
