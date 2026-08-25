use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Object)]
pub struct DisplayEvent {
    pub current_playlist_id: Option<String>,
    pub current_tab_id: Option<String>,
    pub auto_rotate: bool,
    pub screen_on: bool,
}
