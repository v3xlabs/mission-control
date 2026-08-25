use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::config::{HumanDuration, Playlist};

use super::{ApiError, ApiResult};

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CreatePlaylistRequest {
    pub playlist_id: String,
    pub name: Option<String>,
    /// A duration such as `30s`, `5m` or `1h`.
    pub interval: String,
    pub hold: Option<String>,
}

impl CreatePlaylistRequest {
    pub fn into_playlist(self) -> ApiResult<Playlist> {
        Ok(Playlist {
            playlist_id: self.playlist_id,
            name: self.name,
            interval: parse(&self.interval)?,
            hold: self.hold.as_deref().map(parse).transpose()?,
            is_default: false,
            tabs: Vec::new(),
            disabled_tabs: Vec::new(),
        })
    }
}

fn parse(text: &str) -> ApiResult<HumanDuration> {
    HumanDuration::parse(text).map_err(ApiError::bad_request)
}
