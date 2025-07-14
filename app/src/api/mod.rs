use std::sync::Arc;

use poem_openapi::{payload::Json, OpenApi, OpenApiService};
use crate::state::AppState;

pub mod models;
use models::*;

#[derive(Clone)]
pub struct ManagementApi {
    state: Arc<AppState>,
}

impl ManagementApi {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[OpenApi]
impl ManagementApi {
    /// Get all playlists configured on the device.
    #[oai(path = "/playlists", method = "get")]
    async fn get_playlists(&self) -> Json<Vec<PlaylistInfo>> {
        let playlists = self.get_playlists_impl().await.unwrap_or_default();
        Json(playlists)
    }

    /// Get all tabs from a given playlist.
    #[oai(path = "/playlists/:playlist_id/tabs", method = "get")]
    async fn get_playlist_tabs(&self, playlist_id: poem_openapi::param::Path<String>) -> Json<Vec<TabInfo>> {
        let tabs = self
            .get_playlist_tabs_impl(&playlist_id.0)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        Json(tabs)
    }

    /// Retrieve basic device status information.
    #[oai(path = "/status", method = "get")]
    async fn get_status(&self) -> Json<DeviceStatus> {
        let status = self
            .get_status_impl()
            .await
            .unwrap_or_else(|_| DeviceStatus {
                device_id: "unknown".into(),
                device_name: "unknown".into(),
                current_playlist: None,
                current_tab: None,
                uptime_seconds: 0,
            });
        Json(status)
    }
}

impl ManagementApi {
    async fn get_playlists_impl(&self) -> anyhow::Result<Vec<PlaylistInfo>> {
        let mut playlists = Vec::new();

        if let Some(chromium_config) = &self.state.config.chromium {
            if let Some(playlist_configs) = &chromium_config.playlists {
                for (name, config) in playlist_configs {
                    let tab_count = config.tabs.len();
                    let is_active = {
                        let current_playlist = self.state.chrome.current_playlist.lock().await;
                        current_playlist.as_ref() == Some(name)
                    };

                    playlists.push(PlaylistInfo {
                        id: name.clone(),
                        name: name.clone(),
                        tab_count,
                        interval_seconds: config.interval,
                        is_active,
                    });
                }
            }
        }

        Ok(playlists)
    }

    async fn get_playlist_tabs_impl(
        &self,
        playlist_id: &str,
    ) -> anyhow::Result<Option<Vec<TabInfo>>> {
        if let Some(chromium_config) = &self.state.config.chromium {
            if let Some(playlist_configs) = &chromium_config.playlists {
                if let Some(playlist_config) = playlist_configs.get(playlist_id) {
                    let mut tabs = Vec::new();

                    for (index, tab_id) in playlist_config.tabs.iter().enumerate() {
                        if let Some(tab_configs) = &chromium_config.tabs {
                            if let Some(tab_config) = tab_configs.get(tab_id) {
                                tabs.push(TabInfo {
                                    id: tab_id.clone(),
                                    name: tab_id.clone(),
                                    url: tab_config.url.clone(),
                                    order_index: index,
                                    persist: tab_config.persist,
                                });
                            }
                        }
                    }

                    return Ok(Some(tabs));
                }
            }
        }

        Ok(None)
    }

    async fn get_status_impl(&self) -> anyhow::Result<DeviceStatus> {
        let current_playlist = self.state.chrome.current_playlist.lock().await.clone();

        // NOTE: Currently we don't track the precise active tab index. We'll return the first tab.
        let current_tab = if let Some(ref playlist_id) = current_playlist {
            if let Some(chromium_config) = &self.state.config.chromium {
                if let Some(playlist_configs) = &chromium_config.playlists {
                    if let Some(playlist_config) = playlist_configs.get(playlist_id) {
                        playlist_config.tabs.first().cloned()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(DeviceStatus {
            device_id: self.state.config.device.id.clone(),
            device_name: self.state.config.device.name.clone(),
            current_playlist,
            current_tab,
            uptime_seconds: 0, // TODO: Calculate uptime
        })
    }
}

/// Helper to create an `OpenApiService` from the management API
pub fn create_api_service(state: Arc<AppState>) -> OpenApiService<ManagementApi, ()> {
    OpenApiService::new(
        ManagementApi::new(state),
        "Mission Control API",
        "0.1.0",
    )
    .server("/")
} 