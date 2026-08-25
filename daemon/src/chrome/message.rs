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
    /// Put an alert on screen and hold it there until it expires.
    Takeover {
        tab_id: Option<String>,
        stinger: Option<String>,
        seconds: u64,
    },
    /// Return to whatever the playlist was showing before a takeover.
    EndTakeover,
    Shutdown,
}
