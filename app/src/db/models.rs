use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub interval_seconds: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tab {
    pub id: String,
    pub name: String,
    pub url: String,
    pub persist: bool,
    pub viewport_width: Option<i32>,
    pub viewport_height: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlaylistTab {
    pub playlist_id: String,
    pub tab_id: String,
    pub order_index: i64,
    pub duration_seconds: Option<i64>,
    pub enabled: bool,
    pub last_manual_activation: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

// Combined model for playlist with tabs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistWithTabs {
    pub id: String,
    pub name: String,
    pub interval_seconds: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tabs: Vec<TabWithOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabWithOrder {
    pub id: String,
    pub name: String,
    pub url: String,
    pub persist: bool,
    pub viewport_width: Option<i32>,
    pub viewport_height: Option<i32>,
    pub order_index: i64,
    pub duration_seconds: Option<i64>,
    pub enabled: bool,
    pub last_manual_activation: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response models for API
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CreatePlaylistRequest {
    pub id: String,
    pub name: String,
    pub interval_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub interval_seconds: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CreateTabRequest {
    pub id: String,
    pub name: String,
    pub url: String,
    pub persist: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdateTabRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub persist: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AddTabToPlaylistRequest {
    pub tab_id: String,
    pub order_index: i64,
    pub duration_seconds: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ReorderTabsRequest {
    pub tab_orders: Vec<TabOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct TabOrder {
    pub tab_id: String,
    pub order_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ToggleTabEnabledRequest {
    pub enabled: bool,
}
