use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::config::Persisted;

/// What a mutation answers with. `persisted` is false when the config directory is managed
/// elsewhere, so the change applies to the running display and lasts until the next restart.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct MutationResult {
    pub persisted: bool,
}

impl MutationResult {
    pub const fn applied() -> Self {
        Self { persisted: true }
    }
}

impl From<Persisted> for MutationResult {
    fn from(persisted: Persisted) -> Self {
        Self {
            persisted: persisted == Persisted::ToDisk,
        }
    }
}
