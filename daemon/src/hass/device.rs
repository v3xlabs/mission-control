use serde::Serialize;

/// The device every entity belongs to, so Home Assistant groups them under one card.
#[derive(Debug, Clone, Serialize)]
pub struct HassDevice {
    pub identifiers: Vec<String>,
    pub name: String,
    pub serial_number: String,
}
