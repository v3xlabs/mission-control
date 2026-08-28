use serde::{Deserialize, Serialize};

use super::{HumanDuration, SecretRef};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Calendar {
    pub calendar_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// How often the daemon goes back to the network.
    #[serde(default = "default_refresh")]
    pub refresh: HumanDuration,
    /// How far ahead the rail reaches, and therefore how far a recurrence is expanded.
    #[serde(default = "default_window")]
    pub window: HumanDuration,
    /// How long before an entry starts to put a toast over the display. One toast per lead, so
    /// `["5m", "0s"]` says something five minutes out and again as it begins.
    #[serde(default = "default_leads")]
    pub leads: Vec<HumanDuration>,
    /// How long a toast stays up once it appears.
    #[serde(default = "default_toast_duration")]
    pub toast_duration: HumanDuration,
    // Last, because a secret reference serialises as a table and TOML puts every table after the
    // plain values of its parent.
    /// A calendar's `.ics` link is a bearer credential: anyone holding it reads the calendar.
    pub url: SecretRef,
}

fn default_refresh() -> HumanDuration {
    HumanDuration(std::time::Duration::from_secs(15 * 60))
}

fn default_window() -> HumanDuration {
    HumanDuration(std::time::Duration::from_secs(12 * 60 * 60))
}

fn default_leads() -> Vec<HumanDuration> {
    vec![
        HumanDuration(std::time::Duration::from_secs(5 * 60)),
        HumanDuration(std::time::Duration::ZERO),
    ]
}

fn default_toast_duration() -> HumanDuration {
    HumanDuration(std::time::Duration::from_secs(45))
}

impl Calendar {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.calendar_id)
    }
}
