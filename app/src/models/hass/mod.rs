use std::{sync::Arc, time::Duration};

use entity::HassEntity;
use reqwest::Url;
use rumqttc::{Client, Connection, Event, LastWill, MqttOptions, Packet, QoS};

use crate::{config::Config, state::AppState};

pub mod entity;

pub struct HassManager {
    // connection: Connection,
    pub mqtt_client: Client,
    pub availability_topic: String,

    pub brightness_entity: HassEntity,
    pub backlight_entity: HassEntity,
    pub playlist_entity: HassEntity,
    pub tab_entity: HassEntity,
    pub url_entity: HassEntity,
}

impl HassManager {
    pub async fn new(config: &Config) -> (Self, Connection) {
        let mqtt_url = config.homeassistant.mqtt_url.parse::<Url>().unwrap();
        let mqtt_port = mqtt_url.port().unwrap_or(1883);

        let mqtt_url = mqtt_url.host_str().unwrap();

        println!("MQTT URL: {}", mqtt_url);

        let availability_topic = format!("homeassistant/device/{}/availability", config.device.id);
        let availability_lastwill = "offline";

        let mut mqttoptions = MqttOptions::new(
            format!("mission-control-{}", config.device.id),
            mqtt_url,
            mqtt_port,
        );
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

        let brightness_entity = HassEntity::new_brightness(
            config.device.name.to_string(),
            config.device.id.to_string(),
            availability_topic.to_string(),
            Some(|_state, new_state| {
                println!("Brightness state changed: {}", new_state);
            }),
        );

        let backlight_entity = HassEntity::new_backlight(
            config.device.name.to_string(),
            config.device.id.to_string(),
            availability_topic.to_string(),
            Some(|state, new_state| {
                println!("Backlight state changed: {}", new_state);

                if let Some(_xrandr) = &state.config.display.xrandr {
                    if new_state.eq("ON") {
                        let xrandr_command =
                            format!("xset dpms force on && xset s off && xset -dpms",);
                        let xrandr_result = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(xrandr_command)
                            .output()
                            .expect("Failed to execute xrandr command");
                        println!("xrandr result: {:?}", xrandr_result);
                    } else {
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
            }),
        );

        let playlist_options = config.chromium.as_ref().map(|chromium| {
            chromium
                .playlists
                .iter()
                .flat_map(|playlist| {
                    playlist
                        .iter()
                        .map(|(name, _)| name.to_string())
                        .collect::<Vec<String>>()
                })
                .collect::<Vec<String>>()
        });

        let playlist_entity = HassEntity::new_playlist(
            config.device.name.to_string(),
            config.device.id.to_string(),
            availability_topic.to_string(),
            playlist_options,
            Some(|state, new_state| {
                println!("Playlist state changed: {}", new_state);
                let new_state = new_state.to_string();
            }),
        );

        let tab_entity = HassEntity::new_tab(
            config.device.name.to_string(),
            config.device.id.to_string(),
            availability_topic.to_string(),
        );

        let url_entity = HassEntity::new_url(
            config.device.name.to_string(),
            config.device.id.to_string(),
            availability_topic.to_string(),
        );

        (
            Self {
                mqtt_client: client,
                // connection,
                availability_topic,
                brightness_entity,
                backlight_entity,
                playlist_entity,
                tab_entity,
                url_entity,
            },
            connection,
        )
    }

    pub async fn init(&self) {
        self.mqtt_client
            .publish(&self.availability_topic, QoS::AtLeastOnce, true, "online")
            .unwrap();

        self.brightness_entity.publish_config(&self.mqtt_client);
        self.backlight_entity.publish_config(&self.mqtt_client);
        self.playlist_entity.publish_config(&self.mqtt_client);
        self.tab_entity.publish_config(&self.mqtt_client);
        // self.url_entity.publish_config(&self.mqtt_client);

        self.brightness_entity.subscribe(&self.mqtt_client);
        self.backlight_entity.subscribe(&self.mqtt_client);
        self.playlist_entity.subscribe(&self.mqtt_client);
        self.tab_entity.subscribe(&self.mqtt_client);
        // self.url_entity.subscribe(&self.mqtt_client);
    }

    pub async fn run(&self, connection: &mut Connection, state: &Arc<AppState>) {
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

                                    if publish.topic.eq(&self.brightness_entity.command_topic) {
                                        println!("Command received: {:?}", &publish.payload);
                                        self.brightness_entity.handle_command(
                                            &self.mqtt_client,
                                            &state,
                                            &publish.payload,
                                        );
                                    }

                                    if publish.topic.eq(&self.backlight_entity.command_topic) {
                                        println!("Command received: {:?}", &publish.payload);
                                        self.backlight_entity.handle_command(
                                            &self.mqtt_client,
                                            &state,
                                            &publish.payload,
                                        );
                                    }

                                    if publish.topic.eq(&self.playlist_entity.command_topic) {
                                        println!("Command received: {:?}", &publish.payload);
                                        self.playlist_entity.handle_command(
                                            &self.mqtt_client,
                                            &state,
                                            &publish.payload,
                                        );
                                        let payload = String::from_utf8_lossy(&publish.payload).to_string();
                                        state.chrome.set_playlist(payload).await;
                                    }

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
