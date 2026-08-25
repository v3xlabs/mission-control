use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::config::Playlist;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PlaylistInfo {
    pub playlist_id: String,
    pub name: String,
    pub tab_count: usize,
    pub interval: String,
    pub is_active: bool,
    pub is_default: bool,
}

impl PlaylistInfo {
    pub fn new(playlist: &Playlist, tab_count: usize, active: Option<&str>) -> Self {
        Self {
            playlist_id: playlist.playlist_id.clone(),
            name: playlist.display_name().to_string(),
            tab_count,
            interval: playlist.interval.to_string(),
            is_active: active == Some(playlist.playlist_id.as_str()),
            is_default: playlist.is_default,
        }
    }
}
