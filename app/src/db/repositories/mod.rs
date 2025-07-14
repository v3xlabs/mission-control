pub mod playlist;
pub mod tab;
pub mod playlist_tab;

use anyhow::Result;
use async_trait::async_trait;

use crate::db::models::*;

#[async_trait]
pub trait PlaylistRepository {
    async fn create(&self, request: CreatePlaylistRequest) -> Result<Playlist>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Playlist>>;
    async fn get_all(&self) -> Result<Vec<Playlist>>;
    async fn get_with_tabs(&self, id: &str) -> Result<Option<PlaylistWithTabs>>;
    async fn get_all_with_tabs(&self) -> Result<Vec<PlaylistWithTabs>>;
    async fn update(&self, id: &str, request: UpdatePlaylistRequest) -> Result<Option<Playlist>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn set_active(&self, id: &str, is_active: bool) -> Result<()>;
    async fn get_active(&self) -> Result<Option<Playlist>>;
    async fn get_tabs(&self, playlist_id: &str) -> Result<Vec<Tab>>;
    async fn update_interval(&self, playlist_id: &str, interval_seconds: i64) -> Result<()>;
}

#[async_trait]
pub trait TabRepository {
    async fn create(&self, request: CreateTabRequest) -> Result<Tab>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Tab>>;
    async fn get_all(&self) -> Result<Vec<Tab>>;
    async fn update(&self, id: &str, request: UpdateTabRequest) -> Result<Option<Tab>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn update_url(&self, id: &str, url: &str) -> Result<()>;
}

#[async_trait]
pub trait PlaylistTabRepository {
    async fn add_tab_to_playlist(&self, playlist_id: &str, request: AddTabToPlaylistRequest) -> Result<()>;
    async fn remove_tab_from_playlist(&self, playlist_id: &str, tab_id: &str) -> Result<bool>;
    async fn get_playlist_tabs(&self, playlist_id: &str) -> Result<Vec<TabWithOrder>>;
    async fn reorder_tabs(&self, playlist_id: &str, request: ReorderTabsRequest) -> Result<()>;
} 