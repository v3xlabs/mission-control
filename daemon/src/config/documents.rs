use super::{
    CalendarsDocument, DeviceDocument, DisplayDocument, NotificationsDocument, PlaylistsDocument,
    TabsDocument,
};

#[derive(Debug, Clone)]
pub struct Documents {
    pub device: DeviceDocument,
    pub display: DisplayDocument,
    pub tabs: TabsDocument,
    pub playlists: PlaylistsDocument,
    pub notifications: NotificationsDocument,
    pub calendars: CalendarsDocument,
}
