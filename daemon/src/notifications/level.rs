use poem_openapi::Enum;
use serde::{Deserialize, Serialize};

/// How loudly an alert asks to be looked at.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, Enum)]
#[serde(rename_all = "lowercase")]
#[oai(rename_all = "lowercase")]
pub enum Level {
    #[default]
    Info,
    Warning,
    Critical,
}
