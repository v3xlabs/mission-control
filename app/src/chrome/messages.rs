use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChromeMessage {
    /// Activate a specific playlist
    ActivatePlaylist { playlist_id: String },
    /// Activate a specific tab immediately
    ActivateTab { tab_id: String, playlist_id: String },
    /// Stop the current playlist
    StopPlaylist,
    /// Start automatic playlist rotation
    StartPlaylist,
    /// Update playlist interval
    UpdateInterval { playlist_id: String, interval_seconds: i64 },
    /// Reload current tab
    ReloadTab,
    /// Navigate to next tab in playlist
    NextTab,
    /// Navigate to previous tab in playlist
    PreviousTab,
    /// Update tab URL
    UpdateTabUrl { tab_id: String, url: String },
    /// Close tab
    CloseTab { tab_id: String },
    /// Refresh tab (reload page)
    RefreshTab { tab_id: String },
    /// Recreate tab (close and reopen)
    RecreateTab { tab_id: String },
    /// Check Chrome status
    GetStatus,
    /// Shutdown Chrome controller
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChromeResponse {
    /// Operation completed successfully
    Success,
    /// Operation failed with error message
    Error { message: String },
    /// Chrome status response
    Status {
        current_playlist_id: Option<String>,
        current_tab_id: Option<String>,
        is_running: bool,
        auto_rotate: bool,
    },
}

#[derive(Debug, Clone)]
pub struct ChromeState {
    pub current_playlist_id: Option<String>,
    pub current_tab_id: Option<String>,
    pub is_running: bool,
    pub auto_rotate: bool,
    pub current_tab_index: usize,
    pub current_tab_opened_at: Option<std::time::SystemTime>,
}

impl Default for ChromeState {
    fn default() -> Self {
        Self {
            current_playlist_id: None,
            current_tab_id: None,
            is_running: false,
            auto_rotate: false,
            current_tab_index: 0,
            current_tab_opened_at: None,
        }
    }
} 