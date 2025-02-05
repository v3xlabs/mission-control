use std::{sync::Arc, time::Duration};

use anyhow::Result;
use models::hass::HassDeviceDiscoveryPayload;
use reqwest::Url;
use rumqttc::{Client, Event, LastWill, MqttOptions, Packet, QoS};

pub mod config;
pub mod models;

#[async_std::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let config = config::load_config()?;

    println!("Config: {:?}", config);

    let mqtt_url = config.homeassistant.mqtt_url.parse::<Url>().unwrap();
    let mqtt_port = mqtt_url.port().unwrap_or(1883);

    let availability_topic = format!("homeassistant/device/{}/availability", config.device.id);
    let availability_lastwill = "offline";

    let device_discovery_topic = format!("homeassistant/device/{}/config", config.device.id);

    let discovery_topic = format!("homeassistant/switch/{}/config", config.device.id);
    let command_topic = format!("homeassistant/switch/{}/set", config.device.id);
    let state_topic = format!("homeassistant/switch/{}/state", config.device.id);

    let mqtt_url = mqtt_url.host_str().unwrap();

    println!("MQTT URL: {}", mqtt_url);

    let mut mqttoptions = MqttOptions::new("mission-control-1", mqtt_url, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_last_will(LastWill::new(&availability_topic, availability_lastwill, QoS::AtMostOnce, true));

    if let Some(username) = config.homeassistant.mqtt_username {
        if let Some(password) = config.homeassistant.mqtt_password {
            println!("Setting credentials for MQTT connection {} {}", username, password);
            mqttoptions.set_credentials(username, password);
        }
    }

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    client.subscribe(&command_topic, QoS::AtMostOnce).unwrap();

    let client_arc = Arc::new(client);
    let client_arc_2 = client_arc.clone();
    let command_topic_2 = command_topic.clone();
    let state_topic_2 = state_topic.clone();
    // spawn task
    async_std::task::spawn(async move {
        let discovery_payload = HassDeviceDiscoveryPayload::new(
            config.device.name.to_string(),
            config.device.id.to_string(),
            state_topic.to_string(),
            command_topic.to_string(),
            availability_topic.to_string(),
        );
        let discovery_payload_str: String = serde_json::to_string(&discovery_payload).unwrap();
        client_arc.publish(&discovery_topic, QoS::AtLeastOnce, true, discovery_payload_str).unwrap();

        client_arc.publish(&availability_topic, QoS::AtLeastOnce, true, "online").unwrap();

        client_arc.publish(&state_topic, QoS::AtLeastOnce, true, "OFF").unwrap();

        let discovery_payload_device_str: String = serde_json::to_string(&discovery_payload.device).unwrap();
        client_arc.publish(&device_discovery_topic, QoS::AtLeastOnce, true, discovery_payload_device_str).unwrap();
    });

    for (i, notification) in connection.iter().enumerate() {
        println!("Notification: {:?}", notification);

        if let Ok(payload) = notification {
            match payload {
                Event::Incoming(event) => {
                    match event {
                        Packet::Publish(publish) => {
                            println!("Publish: {:?}", &publish);

                            if publish.topic.eq(&command_topic_2) {
                                println!("Command received: {:?}", &publish.payload);

                                if publish.payload.eq("ON") {
                                    println!("Turning on display");
                                    client_arc_2.publish(&state_topic_2, QoS::AtLeastOnce, true, "ON").unwrap();
                                } else if publish.payload.eq("OFF") {
                                    println!("Turning off display");
                                    client_arc_2.publish(&state_topic_2, QoS::AtLeastOnce, true, "OFF").unwrap();
                                }
                            }
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
         }
    }

    Ok(())
}
