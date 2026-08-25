use std::time::{Instant, SystemTime};

#[derive(Debug, Clone, Default)]
pub struct ChromeState {
    pub current_playlist_id: Option<String>,
    pub current_tab_id: Option<String>,
    pub is_running: bool,
    pub auto_rotate: bool,
    pub current_tab_opened_at: Option<SystemTime>,
    pub hold_until: Option<Instant>,
    /// What the playlist was showing before an alert took the screen, so ending the takeover
    /// returns to it rather than to wherever rotation happens to have reached.
    pub interrupted_tab_id: Option<String>,
}
