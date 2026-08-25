#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChromeResponse {
    Success,
    Error {
        message: String,
    },
    Status {
        current_playlist_id: Option<String>,
        current_tab_id: Option<String>,
        is_running: bool,
        auto_rotate: bool,
    },
}
