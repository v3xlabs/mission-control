use serde::{Deserialize, Serialize};

use super::{version, Playlist};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistsDocument {
    #[serde(default = "version::current")]
    pub version: u32,
    #[serde(default)]
    pub playlists: Vec<Playlist>,
}

impl Default for PlaylistsDocument {
    fn default() -> Self {
        Self {
            version: version::CURRENT,
            playlists: Vec::new(),
        }
    }
}
