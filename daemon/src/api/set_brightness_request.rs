use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SetBrightnessRequest {
    /// Percent, 0 through 100.
    pub percent: u32,
}
