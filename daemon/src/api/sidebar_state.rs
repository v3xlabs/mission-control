use poem_openapi::Object;
use serde::{Deserialize, Serialize};

/// Where the rail ended up, so a caller that toggled it does not have to ask again.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SidebarState {
    pub open: bool,
}
