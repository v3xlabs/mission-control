use std::sync::Arc;

use anyhow::Result;

use crate::{
    chrome::ChromeController,
    config::{ConfigStore, Dirs},
    db::Runtime,
    display::Display,
    events::Events,
    hass::HassManager,
};

pub type State = Arc<AppState>;

pub struct AppState {
    pub chrome: Arc<ChromeController>,
    pub config: Arc<ConfigStore>,
    pub display: Arc<Display>,
    pub events: Arc<Events>,
    pub hass: Arc<HassManager>,
    pub runtime: Runtime,
    pub admin_key: Option<String>,
    pub started_at: std::time::Instant,
}

impl AppState {
    pub async fn new(dirs: Dirs) -> Result<Arc<Self>> {
        dirs.create_state_and_cache()?;

        let runtime = Runtime::open(&dirs.state).await?;
        let config = Arc::new(ConfigStore::load(dirs)?);
        let device = config.read().await.device;

        let admin_key = match device.admin_key.as_ref() {
            Some(reference) => Some(reference.resolve()?),
            None => None,
        };

        let hass = HassManager::new(&device).await?;

        Ok(Arc::new(Self {
            chrome: Arc::new(ChromeController::new()),
            config,
            display: Arc::new(Display::new()),
            events: Arc::new(Events::new()),
            hass: Arc::new(hass),
            runtime,
            admin_key,
            started_at: std::time::Instant::now(),
        }))
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}
