#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Document {
    Device,
    Display,
    Tabs,
    Playlists,
}

impl Document {
    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Device => "device.toml",
            Self::Display => "display.toml",
            Self::Tabs => "tabs.toml",
            Self::Playlists => "playlists.toml",
        }
    }
}
