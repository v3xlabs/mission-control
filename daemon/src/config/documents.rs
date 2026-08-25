use super::{DeviceDocument, DisplayDocument, PlaylistsDocument, TabsDocument};

#[derive(Debug, Clone)]
pub struct Documents {
    pub device: DeviceDocument,
    pub display: DisplayDocument,
    pub tabs: TabsDocument,
    pub playlists: PlaylistsDocument,
}
