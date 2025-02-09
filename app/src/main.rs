use std::sync::Arc;

use anyhow::Result;
use async_std::task;
use state::AppState;

pub mod chrome;
pub mod config;
pub mod http;
pub mod models;
pub mod state;

#[async_std::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let config = config::load_config()?;

    println!("Config: {:?}", config);

    let (state, mut connection) = AppState::new(config).await;
    let state = Arc::new(state);
    let chromium = state.chrome.clone();

    if let Some(chromium_config) = &state.config.chromium {
        if chromium_config.enabled {
            let chromium_config_clone = chromium_config.clone();
            let state_clone = state.clone();

            let chromium_2 = chromium.clone();

            task::spawn(async move {
                chromium_2.start(&chromium_config_clone, &state_clone).await.unwrap();
            });
        }
    }

    // let device_discovery_topic = format!("homeassistant/device/{}/config", config.device.id);

    // let discovery_payload = HassDeviceDiscoveryPayload::new_backlight(
    //     config.device.name.to_string(),
    //     config.device.id.to_string(),
    //     state.hass.availability_topic.to_string(),
    // );

    // let discovery_payload_str: String = serde_json::to_string(&discovery_payload).unwrap();
    // let discovery_payload_arc = Arc::new(discovery_payload);
    // let discovery_payload_brightness_str: String =
    // serde_json::to_string(&discovery_payload_brightness).unwrap();
    // let discovery_payload_brightness_arc = Arc::new(discovery_payload_brightness);

    // post mqtt init

    // state.hass.mqtt_client
    //     .subscribe(&discovery_payload_arc.command_topic, QoS::AtMostOnce)
    //     .unwrap();
    // state.hass.mqtt_client
    //     .subscribe(
    //         &discovery_payload_brightness_arc.command_topic,
    //         QoS::AtMostOnce,
    //     )
    //     .unwrap();

    // let discovery_payload_arc_2 = discovery_payload_arc.clone();
    // let discovery_payload_brightness_arc_2 = discovery_payload_brightness_arc.clone();

    state.hass.init().await;

    // spawn task
    // async_std::task::spawn(async move {
    //     state2.hass.mqtt_client
    //         .publish(
    //             &discovery_payload_arc_2.config_topic,
    //             QoS::AtLeastOnce,
    //             true,
    //             discovery_payload_str,
    //         )
    //         .unwrap();
    //     state2.hass.mqtt_client
    //         .publish(
    //             &discovery_payload_brightness_arc_2.config_topic,
    //             QoS::AtLeastOnce,
    //             true,
    //             discovery_payload_brightness_str,
    //         )
    //         .unwrap();

    //     state2.hass.mqtt_client
    //         .publish(&state2.hass.availability_topic, QoS::AtLeastOnce, true, "online")
    //         .unwrap();

    //     state2.hass.mqtt_client
    //         .publish(
    //             &discovery_payload_arc_2.state_topic,
    //             QoS::AtLeastOnce,
    //             true,
    //             "OFF",
    //         )
    //         .unwrap();
    //     state2.hass.mqtt_client
    //         .publish(
    //             &discovery_payload_brightness_arc_2.state_topic,
    //             QoS::AtLeastOnce,
    //             true,
    //             "0.5",
    //         )
    //         .unwrap();

    //     let discovery_payload_device_str: String =
    //         serde_json::to_string(&discovery_payload_arc_2.device).unwrap();
    //     state2.hass.mqtt_client
    //         .publish(
    //             &device_discovery_topic,
    //             QoS::AtLeastOnce,
    //             true,
    //             discovery_payload_device_str,
    //         )
    //         .unwrap();
    // });

    let http_state = state.clone();
    task::spawn(async move {
        http::start_http(http_state).await.unwrap();
    });

    state.hass.run(&mut connection, &state);

    Ok(())
}
