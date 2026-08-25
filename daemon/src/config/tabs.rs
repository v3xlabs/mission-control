use serde::{Deserialize, Serialize};

use super::{version, Tab};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TabsDocument {
    #[serde(default = "version::current")]
    pub version: u32,
    #[serde(default)]
    pub tabs: Vec<Tab>,
}

impl Default for TabsDocument {
    fn default() -> Self {
        Self {
            version: version::CURRENT,
            tabs: Vec::new(),
        }
    }
}
