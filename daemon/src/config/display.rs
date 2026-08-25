use serde::{Deserialize, Serialize};

use super::{version, ScheduleWindow};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplayDocument {
    #[serde(default = "version::current")]
    pub version: u32,
    /// Substituted for `{output}` in the commands below.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default = "default_power_on")]
    pub power_on: Vec<String>,
    #[serde(default = "default_power_off")]
    pub power_off: Vec<String>,
    /// Receives `{percent}`, 0 through 100.
    #[serde(default = "default_brightness")]
    pub brightness: Vec<String>,
    /// Writes the output's current contents to stdout. `grim` speaks wlr-screencopy, which is
    /// what niri implements.
    #[serde(default = "default_screenshot")]
    pub screenshot: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schedule: Vec<ScheduleWindow>,
}

fn default_power_on() -> Vec<String> {
    ["niri", "msg", "action", "power-on-monitors"]
        .map(String::from)
        .to_vec()
}

fn default_power_off() -> Vec<String> {
    ["niri", "msg", "action", "power-off-monitors"]
        .map(String::from)
        .to_vec()
}

fn default_screenshot() -> Vec<String> {
    ["grim", "-t", "jpeg", "-q", "80", "-"]
        .map(String::from)
        .to_vec()
}

fn default_brightness() -> Vec<String> {
    ["ddcutil", "setvcp", "10", "{percent}"]
        .map(String::from)
        .to_vec()
}

impl Default for DisplayDocument {
    fn default() -> Self {
        Self {
            version: version::CURRENT,
            output: None,
            power_on: default_power_on(),
            power_off: default_power_off(),
            brightness: default_brightness(),
            screenshot: default_screenshot(),
            schedule: Vec::new(),
        }
    }
}
