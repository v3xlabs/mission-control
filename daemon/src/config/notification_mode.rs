use poem_openapi::Enum;
use serde::{Deserialize, Serialize};

/// How an alert reaches the screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, Enum)]
#[serde(rename_all = "lowercase")]
#[oai(rename_all = "lowercase")]
pub enum NotificationMode {
    /// The alert becomes what the display shows, and the playlist resumes when it expires.
    #[default]
    Takeover,
    /// The alert opens as its own window beside the content and the compositor lays them out.
    Sidebar,
}
