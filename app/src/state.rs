use std::sync::Arc;
use sqlx::SqlitePool;

use rumqttc::Connection;

use crate::{
    chrome::ChromeController, 
    config::Config, 
    models::hass::HassManager,
    db::repositories::{
        playlist::SqlitePlaylistRepository,
        tab::SqliteTabRepository,
        playlist_tab::SqlitePlaylistTabRepository,
    }
};

pub type State = Arc<AppState>;

pub struct AppState {
    pub chrome: Arc<ChromeController>,
    pub hass: Arc<HassManager>,
    pub config: Config,
    pub db_pool: SqlitePool,
    pub playlist_repository: Arc<SqlitePlaylistRepository>,
    pub tab_repository: Arc<SqliteTabRepository>,
    pub playlist_tab_repository: Arc<SqlitePlaylistTabRepository>,
}

impl AppState {
    pub async fn new(config: Config, db_pool: SqlitePool) -> (Self, Connection) {
        let (hass, connection) = HassManager::new(&config).await;
        let hass = Arc::new(hass);
        let chrome = Arc::new(ChromeController::new());

        // Initialize repositories
        let playlist_repo = Arc::new(SqlitePlaylistRepository::new(db_pool.clone()));
        let tab_repo = Arc::new(SqliteTabRepository::new(db_pool.clone()));
        let playlist_tab_repo = Arc::new(SqlitePlaylistTabRepository::new(db_pool.clone()));

        (
            Self {
                chrome,
                hass,
                config,
                db_pool,
                playlist_repository: playlist_repo,
                tab_repository: tab_repo,
                playlist_tab_repository: playlist_tab_repo,
            },
            connection,
        )
    }
}
