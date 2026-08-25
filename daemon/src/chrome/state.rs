use std::time::{Instant, SystemTime};

#[derive(Debug, Clone, Default)]
pub struct ChromeState {
    pub current_playlist_id: Option<String>,
    pub current_tab_id: Option<String>,
    pub is_running: bool,
    pub auto_rotate: bool,
    pub current_tab_opened_at: Option<SystemTime>,
    pub hold_until: Option<Instant>,
}
