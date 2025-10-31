use std::{sync::Arc, time::Duration};

use crate::{
    chrome::{send_chrome_message, ChromeMessage},
    config::Config,
    db::models::TabWithOrder,
    display,
    state::{AppState, State},
};
use entity::HassEntity;

use reqwest::Url;
use rumqttc::{Client, Connection, Event, LastWill, MqttOptions, Packet, QoS};
use tracing::{info, warn};

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
    pub fn disabled() -> Self {
        // Create a dummy MQTT client that won't be used
        let mut mqttoptions = MqttOptions::new("disabled", "localhost", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(60));
        let (mqtt_client, _) = Client::new(mqttoptions, 10);

        Self {
            mqtt_client,
            availability_topic: String::new(),
            brightness_entity: HassEntity::new_brightness(
                String::new(),
                String::new(),
                String::new(),
                None,
            ),
            backlight_entity: HassEntity::new_backlight(
                String::new(),
                String::new(),
                String::new(),
                None,
            ),
            playlist_entity: HassEntity::new_playlist(
                String::new(),
                String::new(),
                String::new(),
                None,
                None,
            ),
            tab_entity: HassEntity::new_tab(String::new(), String::new(), String::new()),
            url_entity: HassEntity::new_url(String::new(), String::new(), String::new()),
        }
    }

    pub fn publish_playlist_options(&self, playlists: Vec<String>, active_playlist: Option<&str>) {
        let mut entity = self.playlist_entity.clone();
        entity.options = Some(playlists);
        entity.publish_config(&self.mqtt_client);
        if let Some(active) = active_playlist {
            entity.update_state(&self.mqtt_client, active);
        }
    }

    pub fn publish_tab_options(&self, tabs: &[TabWithOrder], active_tab: Option<&str>) {
        let mut entity = self.tab_entity.clone();
        entity.options = Some(tabs.iter().map(|t| t.id.clone()).collect());
        entity.publish_config(&self.mqtt_client);
        if let Some(active) = active_tab {
            entity.update_state(&self.mqtt_client, active);
        }
    }

    pub async fn new(config: &Config) -> (Self, Connection) {
        let hass_config = config
            .homeassistant
            .as_ref()
            .expect("HomeAssistant config required");
        let mqtt_url = hass_config.mqtt.url.parse::<Url>().unwrap();
        let mqtt_port = mqtt_url.port().unwrap_or(1883);

        let mqtt_url = mqtt_url.host_str().unwrap();

        info!("MQTT URL: {}", mqtt_url);

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

        if let Some(username) = &hass_config.mqtt.username {
            if let Some(password) = &hass_config.mqtt.password {
                info!(
                    "Setting credentials for MQTT connection {} {}",
                    username, password
                );
                mqttoptions.set_credentials(username, password);
            }
        }

        let (client, connection) = Client::new(mqttoptions, 10);

        let brightness_entity = HassEntity::new_brightness(
            config.device.name.to_string(),
            config.device.id.to_string(),
            availability_topic.to_string(),
            Some(handle_brightness_change),
        );

        let backlight_entity = HassEntity::new_backlight(
            config.device.name.to_string(),
            config.device.id.to_string(),
            availability_topic.to_string(),
            Some(handle_backlight_change),
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
            Some(|_state, new_state| {
                info!("Playlist state changed: {}", new_state);
                // let new_state = new_state.to_string();
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
        // Skip initialization if MQTT is disabled
        if self.availability_topic.is_empty() {
            return;
        }

        self.mqtt_client
            .publish(&self.availability_topic, QoS::AtLeastOnce, true, "online")
            .unwrap();

        self.brightness_entity.publish_config(&self.mqtt_client);
        self.backlight_entity.publish_config(&self.mqtt_client);
        self.playlist_entity.publish_config(&self.mqtt_client);
        self.tab_entity.publish_config(&self.mqtt_client);
        self.url_entity.publish_config(&self.mqtt_client);

        self.brightness_entity.subscribe(&self.mqtt_client);
        self.backlight_entity.subscribe(&self.mqtt_client);
        self.playlist_entity.subscribe(&self.mqtt_client);
        self.tab_entity.subscribe(&self.mqtt_client);
        // self.url_entity.subscribe(&self.mqtt_client);
    }

    pub async fn run(&self, connection: &mut Connection, state: &Arc<AppState>) {
        let mut _error_count = 0;

        for notification in connection.iter() {
            info!("Notification: {:?}", notification);

            match notification {
                Ok(payload) => {
                    if let Event::Incoming(event) = payload {
                        if let Packet::Publish(publish) = event {
                            info!("Publish: {:?}", &publish);

                            if publish.topic.eq(&self.brightness_entity.command_topic) {
                                info!("Command received: {:?}", &publish.payload);
                                self.brightness_entity.handle_command(
                                    &self.mqtt_client,
                                    state,
                                    &publish.payload,
                                );
                            }

                            if publish.topic.eq(&self.backlight_entity.command_topic) {
                                info!("Command received: {:?}", &publish.payload);
                                self.backlight_entity.handle_command(
                                    &self.mqtt_client,
                                    state,
                                    &publish.payload,
                                );
                            }

                            if publish.topic.eq(&self.playlist_entity.command_topic) {
                                info!("Command received: {:?}", &publish.payload);
                                self.playlist_entity.handle_command(
                                    &self.mqtt_client,
                                    state,
                                    &publish.payload,
                                );
                                let payload = String::from_utf8_lossy(&publish.payload).to_string();
                                let _ = crate::chrome::send_chrome_message(
                                    &state.chrome,
                                    crate::chrome::ChromeMessage::ActivatePlaylist {
                                        playlist_id: payload,
                                    },
                                )
                                .await;
                            }

                            if publish.topic.eq(&self.tab_entity.command_topic) {
                                info!("Command received: {:?}", &publish.payload);
                                self.tab_entity.handle_command(
                                    &self.mqtt_client,
                                    state,
                                    &publish.payload,
                                );
                                let payload = String::from_utf8_lossy(&publish.payload).to_string();
                                if let Some(active_playlist) =
                                    state.chrome.state.lock().await.current_playlist_id.clone()
                                {
                                    let _ = send_chrome_message(
                                        &state.chrome,
                                        ChromeMessage::ActivateTab {
                                            tab_id: payload,
                                            playlist_id: active_playlist,
                                        },
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    _error_count += 1;
                    info!("Error: {:?}", e);

                    // if error_count > 10 {
                    // info!("Too many errors, exiting");
                    // break;
                    // }
                }
            }
        }
    }
}

fn handle_backlight_change(state: &State, new_state: &str) {
    info!("Backlight state changed: {}", new_state);

    let output = state.config.display.output.as_deref();
    let turn_on = new_state.eq_ignore_ascii_case("ON");

    if let Err(err) = display::set_dpms(turn_on, output) {
        warn!("Failed to set DPMS via swaymsg/wlr-randr: {}", err);
    }

    state
        .hass
        .backlight_entity
        .update_state(&state.hass.mqtt_client, new_state);
}

fn handle_brightness_change(state: &State, new_state: &str) {
    info!("Brightness state changed: {}", new_state);

    let Ok(value) = new_state.parse::<f32>() else {
        warn!("Brightness payload not a number: {}", new_state);
        return;
    };

    let display_target = state.config.display.ddcutil_display.as_deref();
    if let Err(err) = display::set_ddc_brightness(display_target, value) {
        warn!("Failed to set brightness via ddcutil: {}", err);
    }

    state
        .hass
        .brightness_entity
        .update_state(&state.hass.mqtt_client, new_state);
}
