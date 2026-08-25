use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ReorderRequest {
    /// The playlist's tabs, in the order they should play. It must hold exactly the tabs the
    /// playlist already has, so a stale browser cannot silently drop one.
    pub tab_ids: Vec<String>,
}
