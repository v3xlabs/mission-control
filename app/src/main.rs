use std::sync::Arc;

use anyhow::Result;
use async_std::task;
use state::AppState;
use tracing::info;

pub mod chrome;
pub mod config;
pub mod http;
pub mod models;
pub mod state;

#[async_std::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Hello, world!");

    let config = config::load_config()?;

    info!("Config: {:?}", config);

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

    state.hass.init().await;

    let http_state = state.clone();
    task::spawn(async move {
        http::start_http(http_state).await.unwrap();
    });

    state.hass.run(&mut connection, &state).await;

    Ok(())
}
