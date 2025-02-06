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

    let discovery_payload = HassDeviceDiscoveryPayload::new_backlight(
        config.device.name.to_string(),
        config.device.id.to_string(),
        availability_topic.to_string(),
    );
    let discovery_payload_brightness = HassDeviceDiscoveryPayload::new_brightness(
        config.device.name.to_string(),
        config.device.id.to_string(),
        availability_topic.to_string(),
    );
    let discovery_payload_str: String = serde_json::to_string(&discovery_payload).unwrap();
    let discovery_payload_arc = Arc::new(discovery_payload);
    let discovery_payload_brightness_str: String =
        serde_json::to_string(&discovery_payload_brightness).unwrap();
    let discovery_payload_brightness_arc = Arc::new(discovery_payload_brightness);

    let mqtt_url = mqtt_url.host_str().unwrap();

    println!("MQTT URL: {}", mqtt_url);

    let mut mqttoptions = MqttOptions::new("mission-control-1", mqtt_url, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_last_will(LastWill::new(
        &availability_topic,
        availability_lastwill,
        QoS::AtMostOnce,
        true,
    ));

    if let Some(username) = config.homeassistant.mqtt_username {
        if let Some(password) = config.homeassistant.mqtt_password {
            println!(
                "Setting credentials for MQTT connection {} {}",
                username, password
            );
            mqttoptions.set_credentials(username, password);
        }
    }

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    client
        .subscribe(&discovery_payload_arc.command_topic, QoS::AtMostOnce)
        .unwrap();
    client
        .subscribe(
            &discovery_payload_brightness_arc.command_topic,
            QoS::AtMostOnce,
        )
        .unwrap();

    let client_arc = Arc::new(client);
    let client_arc_2 = client_arc.clone();

    let discovery_payload_arc_2 = discovery_payload_arc.clone();
    let discovery_payload_brightness_arc_2 = discovery_payload_brightness_arc.clone();

    // spawn task
    async_std::task::spawn(async move {
        client_arc
            .publish(
                &discovery_payload_arc_2.config_topic,
                QoS::AtLeastOnce,
                true,
                discovery_payload_str,
            )
            .unwrap();
        client_arc
            .publish(
                &discovery_payload_brightness_arc_2.config_topic,
                QoS::AtLeastOnce,
                true,
                discovery_payload_brightness_str,
            )
            .unwrap();

        client_arc
            .publish(&availability_topic, QoS::AtLeastOnce, true, "online")
            .unwrap();

        client_arc
            .publish(
                &discovery_payload_arc_2.state_topic,
                QoS::AtLeastOnce,
                true,
                "OFF",
            )
            .unwrap();
        client_arc
            .publish(
                &discovery_payload_brightness_arc_2.state_topic,
                QoS::AtLeastOnce,
                true,
                "0.5",
            )
            .unwrap();

        let discovery_payload_device_str: String =
            serde_json::to_string(&discovery_payload_arc_2.device).unwrap();
        client_arc
            .publish(
                &device_discovery_topic,
                QoS::AtLeastOnce,
                true,
                discovery_payload_device_str,
            )
            .unwrap();
    });

    for (i, notification) in connection.iter().enumerate() {
        println!("Notification: {:?}", notification);

        if let Ok(payload) = notification {
            match payload {
                Event::Incoming(event) => {
                    match event {
                        Packet::Publish(publish) => {
                            println!("Publish: {:?}", &publish);

                            if publish.topic.eq(&discovery_payload_arc.command_topic) {
                                println!("Command received: {:?}", &publish.payload);

                                if publish.payload.eq("ON") {
                                    println!("Turning on display");
                                    client_arc_2
                                        .publish(
                                            &discovery_payload_arc.state_topic,
                                            QoS::AtLeastOnce,
                                            true,
                                            "ON",
                                        )
                                        .unwrap();

                                    if let Some(xrandr) = &config.display.xrandr {
                                        let xrandr_command = format!(
                                            "xset dpms force on && xset s off && xset -dpms",
                                        );
                                        let xrandr_result = std::process::Command::new("sh")
                                            .arg("-c")
                                            .arg(xrandr_command)
                                            .output()
                                            .expect("Failed to execute xrandr command");
                                        println!("xrandr result: {:?}", xrandr_result);
                                    }
                                } else if publish.payload.eq("OFF") {
                                    println!("Turning off display");
                                    client_arc_2
                                        .publish(
                                            &discovery_payload_arc.state_topic,
                                            QoS::AtLeastOnce,
                                            true,
                                            "OFF",
                                        )
                                        .unwrap();

                                    if let Some(xrandr) = &config.display.xrandr {
                                        let xrandr_command = format!(
                                            "xset s off && xset +dpms && xset dpms 600 600 600 && xset dpms force off",
                                        );
                                        let xrandr_result = std::process::Command::new("sh")
                                            .arg("-c")
                                            .arg(xrandr_command)
                                            .output()
                                            .expect("Failed to execute xrandr command");
                                        println!("xrandr result: {:?}", xrandr_result);
                                    }
                                }
                            }

                            if publish
                                .topic
                                .eq(&discovery_payload_brightness_arc.command_topic)
                            {
                                println!("Command received: {:?}", &publish.payload);

                                // Convert bytes to string and parse as f32
                                let brightness_str = String::from_utf8_lossy(&publish.payload);
                                let brightness_value: f32 = brightness_str.parse().unwrap();
                                client_arc_2
                                    .publish(
                                        &discovery_payload_brightness_arc.state_topic,
                                        QoS::AtLeastOnce,
                                        true,
                                        brightness_value.to_string().as_str(),
                                    )
                                    .unwrap();

                                if let Some(xrandr) = &config.display.xrandr {
                                    let xrandr_command = format!(
                                        "xrandr --output {} --brightness {}",
                                        xrandr, brightness_value
                                    );
                                    println!("xrandr command: {}", xrandr_command);
                                    let xrandr_result = std::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(xrandr_command)
                                        .output()
                                        .expect("Failed to execute xrandr command");
                                    println!("xrandr result: {:?}", xrandr_result);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
