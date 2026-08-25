#[derive(Debug, Clone)]
pub enum ChromeMessage {
    ActivatePlaylist { playlist_id: String },
    ActivateTab { tab_id: String, playlist_id: String },
    Pause,
    Resume,
    NextTab,
    PreviousTab,
    RefreshTab { tab_id: String },
    RecreateTab { tab_id: String },
    CloseTab { tab_id: String },
    GetStatus,
    Shutdown,
}
