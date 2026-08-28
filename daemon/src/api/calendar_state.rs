use poem_openapi::Object;
use serde::{Deserialize, Serialize};

/// Whether the full-screen agenda is what the display is showing.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CalendarState {
    pub showing: bool,
}
