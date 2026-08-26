use serde::{Deserialize, Serialize};

use super::SecretRef;

/// Where a tab's content comes from, and therefore what draws it.
///
/// A browser has no `rtsp://` handler, so a camera cannot be a page. Making that a variant rather
/// than a second optional field means a tab is one or the other and never both.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Url(String),
    /// The whole stream URL is a secret, because the credential is part of it.
    Rtsp(SecretRef),
}

impl Source {
    /// What may be shown about this tab. A camera reports its kind rather than its address, so a
    /// credential cannot reach the web UI, Home Assistant or a log line.
    pub fn describe(&self) -> &str {
        match self {
            Self::Url(url) => url,
            Self::Rtsp(_) => "rtsp",
        }
    }
}
