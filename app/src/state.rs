use std::sync::Arc;

use rumqttc::Connection;

use crate::{chrome::ChromeController, config::Config, models::hass::HassManager};

pub type State = Arc<AppState>;

pub struct AppState {
    pub chrome: Arc<ChromeController>,
    pub hass: Arc<HassManager>,
    pub config: Config,
}

impl AppState {
    pub async fn new(config: Config) -> (Self, Connection) {
        let (hass, connection) = HassManager::new(&config).await;
        let hass = Arc::new(hass);
        let chrome = Arc::new(ChromeController::new());

        (
            Self {
                chrome,
                hass,
                config,
            },
            connection,
        )
    }
}
