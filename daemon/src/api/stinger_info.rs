use poem_openapi::Object;
use serde::{Deserialize, Serialize};

/// What the transition page needs: which file to play for a given name. The configured duration
/// stays on the daemon, which is what decides how long the page is up.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct StingerInfo {
    pub name: String,
    pub file: String,
}
