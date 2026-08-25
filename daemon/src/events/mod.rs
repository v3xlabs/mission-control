pub mod display_event;

pub use display_event::DisplayEvent;

use std::sync::Arc;

use tokio::sync::watch;

use crate::state::AppState;

pub struct Events {
    sender: watch::Sender<DisplayEvent>,
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

impl Events {
    pub fn new() -> Self {
        Self {
            sender: watch::channel(DisplayEvent::default()).0,
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<DisplayEvent> {
        self.sender.subscribe()
    }

    pub async fn publish(&self, app_state: &Arc<AppState>) {
        let chrome = app_state.chrome.state.lock().await;

        let event = DisplayEvent {
            current_playlist_id: chrome.current_playlist_id.clone(),
            current_tab_id: chrome.current_tab_id.clone(),
            auto_rotate: chrome.auto_rotate,
            screen_on: app_state.display.is_on(),
        };

        self.sender.send_if_modified(|current| {
            if *current == event {
                false
            } else {
                *current = event;
                true
            }
        });
    }
}
