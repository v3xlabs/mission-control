use std::sync::Arc;

use poem_openapi::{payload::Json, OpenApi, OpenApiService};
use crate::{
    state::AppState,
    db::repositories::{PlaylistRepository, TabRepository, PlaylistTabRepository}
};


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

    /// Activate a playlist
    #[oai(path = "/playlists/:playlist_id/activate", method = "post")]
    async fn activate_playlist(&self, playlist_id: poem_openapi::param::Path<String>) -> poem_openapi::payload::PlainText<String> {
        let pid = playlist_id.0.clone();
        tracing::info!("API: Activating playlist {}", pid);
        
        // Send message to Chrome controller
        if let Err(e) = crate::chrome::send_chrome_message(&self.state.chrome, 
            crate::chrome::ChromeMessage::ActivatePlaylist { playlist_id: pid.clone() }).await {
            tracing::error!("API: Error activating playlist {}: {}", pid, e);
            return poem_openapi::payload::PlainText(format!("Error activating playlist: {}", e));
        }
        
        tracing::info!("API: Successfully sent activate playlist message for {}", pid);
        poem_openapi::payload::PlainText("ok".into())
    }

    /// Activate a tab immediately
    #[oai(path = "/playlists/:playlist_id/tabs/:tab_id/activate", method = "post")]
    async fn activate_tab(&self, playlist_id: poem_openapi::param::Path<String>, tab_id: poem_openapi::param::Path<String>) -> poem_openapi::payload::PlainText<String> {
        let pid = playlist_id.0.clone();
        let tid = tab_id.0.clone();
        tracing::info!("API: Activating tab {} in playlist {}", tid, pid);

        // Send message to Chrome controller to activate tab
        if let Err(e) = crate::chrome::send_chrome_message(&self.state.chrome, 
            crate::chrome::ChromeMessage::ActivateTab { tab_id: tid.clone(), playlist_id: pid.clone() }).await {
            tracing::error!("API: Error activating tab {}: {}", tid, e);
            return poem_openapi::payload::PlainText(format!("Error activating tab: {}", e));
        }

        tracing::info!("API: Successfully sent activate tab message for {}", tid);
        poem_openapi::payload::PlainText("ok".into())
    }

    /// Create a new playlist
    #[oai(path = "/playlists", method = "post")]
    async fn create_playlist(&self, request: Json<crate::db::models::CreatePlaylistRequest>) -> Json<PlaylistInfo> {
        match self.state.playlist_repository.create(request.0).await {
            Ok(playlist) => Json(PlaylistInfo {
                id: playlist.id,
                name: playlist.name,
                tab_count: 0,
                interval_seconds: playlist.interval_seconds,
                is_active: playlist.is_active,
            }),
            Err(e) => {
                // Return error playlist info - in real implementation should return proper error
                Json(PlaylistInfo {
                    id: "error".to_string(),
                    name: format!("Error: {}", e),
                    tab_count: 0,
                    interval_seconds: 30,
                    is_active: false,
                })
            }
        }
    }

    /// Update an existing playlist
    #[oai(path = "/playlists/:playlist_id", method = "put")]
    async fn update_playlist(&self, playlist_id: poem_openapi::param::Path<String>, request: Json<crate::db::models::UpdatePlaylistRequest>) -> Json<PlaylistInfo> {
        match self.state.playlist_repository.update(&playlist_id.0, request.0).await {
            Ok(Some(playlist)) => {
                // Get tab count
                let tabs = self.state.playlist_tab_repository.get_playlist_tabs(&playlist.id).await.unwrap_or_default();
                Json(PlaylistInfo {
                    id: playlist.id,
                    name: playlist.name,
                    tab_count: tabs.len(),
                    interval_seconds: playlist.interval_seconds,
                    is_active: playlist.is_active,
                })
            },
            Ok(None) => Json(PlaylistInfo {
                id: "not_found".to_string(),
                name: "Playlist not found".to_string(),
                tab_count: 0,
                interval_seconds: 30,
                is_active: false,
            }),
            Err(e) => Json(PlaylistInfo {
                id: "error".to_string(),
                name: format!("Error: {}", e),
                tab_count: 0,
                interval_seconds: 30,
                is_active: false,
            })
        }
    }

    /// Delete a playlist
    #[oai(path = "/playlists/:playlist_id", method = "delete")]
    async fn delete_playlist(&self, playlist_id: poem_openapi::param::Path<String>) -> poem_openapi::payload::PlainText<String> {
        match self.state.playlist_repository.delete(&playlist_id.0).await {
            Ok(true) => poem_openapi::payload::PlainText("Playlist deleted successfully".to_string()),
            Ok(false) => poem_openapi::payload::PlainText("Playlist not found".to_string()),
            Err(e) => poem_openapi::payload::PlainText(format!("Error: {}", e)),
        }
    }

    /// Create a new tab
    #[oai(path = "/tabs", method = "post")]
    async fn create_tab(&self, request: Json<crate::db::models::CreateTabRequest>) -> Json<TabInfo> {
        match self.state.tab_repository.create(request.0).await {
            Ok(tab) => Json(TabInfo {
                id: tab.id,
                name: tab.name,
                url: tab.url,
                order_index: 0,
                persist: tab.persist,
            }),
            Err(e) => Json(TabInfo {
                id: "error".to_string(),
                name: format!("Error: {}", e),
                url: "".to_string(),
                order_index: 0,
                persist: false,
            })
        }
    }

    /// Update an existing tab
    #[oai(path = "/tabs/:tab_id", method = "put")]
    async fn update_tab(&self, tab_id: poem_openapi::param::Path<String>, request: Json<crate::db::models::UpdateTabRequest>) -> Json<TabInfo> {
        match self.state.tab_repository.update(&tab_id.0, request.0).await {
            Ok(Some(tab)) => Json(TabInfo {
                id: tab.id,
                name: tab.name,
                url: tab.url,
                order_index: 0,
                persist: tab.persist,
            }),
            Ok(None) => Json(TabInfo {
                id: "not_found".to_string(),
                name: "Tab not found".to_string(),
                url: "".to_string(),
                order_index: 0,
                persist: false,
            }),
            Err(e) => Json(TabInfo {
                id: "error".to_string(),
                name: format!("Error: {}", e),
                url: "".to_string(),
                order_index: 0,
                persist: false,
            })
        }
    }

    /// Delete a tab
    #[oai(path = "/tabs/:tab_id", method = "delete")]
    async fn delete_tab(&self, tab_id: poem_openapi::param::Path<String>) -> poem_openapi::payload::PlainText<String> {
        match self.state.tab_repository.delete(&tab_id.0).await {
            Ok(true) => poem_openapi::payload::PlainText("Tab deleted successfully".to_string()),
            Ok(false) => poem_openapi::payload::PlainText("Tab not found".to_string()),
            Err(e) => poem_openapi::payload::PlainText(format!("Error: {}", e)),
        }
    }

    /// Add a tab to a playlist
    #[oai(path = "/playlists/:playlist_id/tabs", method = "post")]
    async fn add_tab_to_playlist(&self, playlist_id: poem_openapi::param::Path<String>, request: Json<crate::db::models::AddTabToPlaylistRequest>) -> poem_openapi::payload::PlainText<String> {
        match self.state.playlist_tab_repository.add_tab_to_playlist(&playlist_id.0, request.0).await {
            Ok(()) => poem_openapi::payload::PlainText("Tab added to playlist successfully".to_string()),
            Err(e) => poem_openapi::payload::PlainText(format!("Error: {}", e)),
        }
    }

    /// Remove a tab from a playlist
    #[oai(path = "/playlists/:playlist_id/tabs/:tab_id", method = "delete")]
    async fn remove_tab_from_playlist(&self, playlist_id: poem_openapi::param::Path<String>, tab_id: poem_openapi::param::Path<String>) -> poem_openapi::payload::PlainText<String> {
        match self.state.playlist_tab_repository.remove_tab_from_playlist(&playlist_id.0, &tab_id.0).await {
            Ok(true) => poem_openapi::payload::PlainText("Tab removed from playlist successfully".to_string()),
            Ok(false) => poem_openapi::payload::PlainText("Tab not found in playlist".to_string()),
            Err(e) => poem_openapi::payload::PlainText(format!("Error: {}", e)),
        }
    }

    /// Reorder tabs in a playlist
    #[oai(path = "/playlists/:playlist_id/reorder", method = "put")]
    async fn reorder_tabs(&self, playlist_id: poem_openapi::param::Path<String>, request: Json<crate::db::models::ReorderTabsRequest>) -> poem_openapi::payload::PlainText<String> {
        match self.state.playlist_tab_repository.reorder_tabs(&playlist_id.0, request.0).await {
            Ok(()) => poem_openapi::payload::PlainText("Tabs reordered successfully".to_string()),
            Err(e) => poem_openapi::payload::PlainText(format!("Error: {}", e)),
        }
    }
}

impl ManagementApi {
    async fn get_playlists_impl(&self) -> anyhow::Result<Vec<PlaylistInfo>> {
        let playlists_with_tabs = self.state.playlist_repository.get_all_with_tabs().await?;
        
        let mut playlists = Vec::new();
        for playlist in playlists_with_tabs {
            playlists.push(PlaylistInfo {
                id: playlist.id,
                name: playlist.name,
                tab_count: playlist.tabs.len(),
                interval_seconds: playlist.interval_seconds,
                is_active: playlist.is_active,
            });
        }

        Ok(playlists)
    }

    async fn get_playlist_tabs_impl(
        &self,
        playlist_id: &str,
    ) -> anyhow::Result<Option<Vec<TabInfo>>> {
        let tabs_with_order = self.state.playlist_tab_repository.get_playlist_tabs(playlist_id).await?;
        
        let mut tabs = Vec::new();
        for tab in tabs_with_order {
            tabs.push(TabInfo {
                id: tab.id,
                name: tab.name,
                url: tab.url,
                order_index: tab.order_index as usize,
                persist: tab.persist,
            });
        }

        Ok(Some(tabs))
    }

    async fn get_status_impl(&self) -> anyhow::Result<DeviceStatus> {
        let chrome_state = self.state.chrome.state.lock().await;
        let current_playlist = chrome_state.current_playlist_id.clone();
        let current_tab = chrome_state.current_tab_id.clone();

        // Log state for debugging
        tracing::info!("Chrome state - playlist: {:?}, tab: {:?}, running: {}", 
            current_playlist, current_tab, chrome_state.is_running);

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