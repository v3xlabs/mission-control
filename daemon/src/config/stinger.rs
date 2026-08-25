use serde::{Deserialize, Serialize};

use super::HumanDuration;

/// A short clip played while the screen changes.
///
/// The point is not decoration. A camera feed takes seconds to connect, and a viewer watching a
/// blank page reads that as broken. Playing a clip gives the page underneath time to load, so the
/// wait becomes part of the transition rather than a fault.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stinger {
    /// A path under the config directory's `media`, or an absolute path.
    pub file: String,
    /// The clip is cut off here even if it has not ended, so a mis-encoded file cannot strand the
    /// display on a transition.
    #[serde(default = "default_max_duration")]
    pub max_duration: HumanDuration,
}

fn default_max_duration() -> HumanDuration {
    HumanDuration(std::time::Duration::from_secs(5))
}
