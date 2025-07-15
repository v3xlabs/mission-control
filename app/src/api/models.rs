use poem_openapi::Object;
use serde::{Deserialize, Serialize};

/// Information about a playlist
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PlaylistInfo {
    /// Unique identifier for the playlist
    pub id: String,
    /// Display name of the playlist
    pub name: String,
    /// Number of tabs in the playlist
    pub tab_count: usize,
    /// Interval between tab switches in seconds
    pub interval_seconds: i64,
    /// Whether this playlist is currently active
    pub is_active: bool,
}

/// Information about a tab
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct TabInfo {
    /// Unique identifier for the tab
    pub id: String,
    /// Display name of the tab
    pub name: String,
    /// URL the tab displays
    pub url: String,
    /// Order within the playlist (0-based index)
    pub order_index: usize,
    /// Whether this tab persists in browser memory
    pub persist: bool,
    /// Viewport width in pixels (if available)
    pub viewport_width: Option<i32>,
    /// Viewport height in pixels (if available)
    pub viewport_height: Option<i32>,
}

/// Current device status
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DeviceStatus {
    /// Device unique identifier
    pub device_id: String,
    /// Device display name
    pub device_name: String,
    /// Currently active playlist ID (if any)
    pub current_playlist: Option<String>,
    /// Currently active tab ID (if any)
    pub current_tab: Option<String>,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AuthRequest {
    /// Admin key for authentication
    pub admin_key: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AuthResponse {
    /// Whether authentication was successful
    pub success: bool,
    /// Authentication token (if successful)
    pub token: Option<String>,
    /// Error message (if failed)
    pub message: Option<String>,
} 