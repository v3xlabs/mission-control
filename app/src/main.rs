use std::sync::Arc;

use anyhow::Result;
use async_std::task;
use state::AppState;
use tracing::info;

pub mod api;
pub mod chrome;
pub mod config;
pub mod db;
pub mod http;
pub mod models;
pub mod state;

#[async_std::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Hello, world!");

    let config = config::load_config()?;

    info!("Config: {:?}", config);

    // Initialize database
    let db_pool = db::init_database().await?;
    
    // Import config data if chromium config exists
    if let Some(ref chromium_config) = config.chromium {
        db::import_config_data(&db_pool, chromium_config).await?;
    }

    let (state, mut connection) = AppState::new(config, db_pool).await;
    let state = Arc::new(state);

    if let Some(chromium_config) = &state.config.chromium {
        if chromium_config.enabled {
            let chromium_config_clone = chromium_config.clone();
            let state_clone = state.clone();

            task::spawn(async move {
                if let Err(e) = state_clone.chrome.start(&chromium_config_clone, &state_clone).await {
                    tracing::error!("Failed to start Chrome controller: {}", e);
                }
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
