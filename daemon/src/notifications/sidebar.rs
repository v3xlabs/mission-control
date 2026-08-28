use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::warn;

use crate::{niri, state::AppState};

use super::Surface;

/// The agenda and alert rail, in a column beside whatever the playlist is showing.
///
/// Making room is the daemon's job rather than the compositor's. niri scrolls its columns instead
/// of shrinking them, so a rail that is merely opened lands off the side of the output. The
/// display browser is narrowed to the remainder, and put into windowed fullscreen first: Chromium
/// hides its tab strip and its omnibox only while it believes it is fullscreen, and a window that
/// really is fullscreen covers the output rather than sharing it.
pub struct Sidebar {
    surface: Surface,
    /// The display browser as it was before the rail made room, so closing puts it back.
    narrowed: Mutex<Option<Narrowed>>,
}

struct Narrowed {
    window_id: u64,
    /// The width the display had before the rail took some, so closing does not have to ask the
    /// compositor again on a path where the answer would be a warning nobody reads.
    full_width: u32,
    /// Whether the browser was taken out of a real fullscreen to be tiled.
    was_fullscreen: bool,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            surface: Surface::new("sidebar-profile", "notify.html?sidebar=1"),
            narrowed: Mutex::new(None),
        }
    }

    pub async fn is_open(&self) -> bool {
        self.surface.is_running().await
    }

    pub async fn open(&self, app_state: &Arc<AppState>) {
        if self.is_open().await {
            return;
        }

        let config = app_state.config.read().await;
        let width = config.notifications.sidebar_width;

        let window_id = match self.surface.open(app_state).await {
            Ok(window_id) => window_id,
            Err(error) => {
                warn!("could not open the sidebar: {error}");

                return;
            }
        };

        if let Err(error) = self
            .make_room(app_state, width, config.device.chromium.fullscreen)
            .await
        {
            warn!("could not make room for the sidebar: {error}");
        }

        if let Err(error) = niri::set_width(window_id, width).await {
            warn!("could not set the sidebar width: {error}");
        }

        // The content keeps the focus. A focused rail draws the compositor's focus ring around
        // itself and takes the keyboard away from a page that may want it.
        if let Some(narrowed) = self.narrowed.lock().await.as_ref() {
            if let Err(error) = niri::focus_id(narrowed.window_id).await {
                warn!("could not give the display back the focus: {error}");
            }
        }
    }

    pub async fn close(&self) {
        self.surface.close().await;

        let Some(narrowed) = self.narrowed.lock().await.take() else {
            return;
        };

        if let Err(error) = self.give_room_back(&narrowed).await {
            warn!("could not give the display its width back: {error}");
        }
    }

    async fn make_room(
        &self,
        app_state: &Arc<AppState>,
        width: u32,
        fullscreen: bool,
    ) -> anyhow::Result<()> {
        let Some(pid) = app_state.chrome.browser_pid().await else {
            return Ok(());
        };

        let window_id = niri::wait_for_pid(pid).await?.id;
        let output = app_state.config.read().await.display.output;
        let total = niri::output_width(output.as_deref()).await?;

        if fullscreen {
            niri::toggle_windowed_fullscreen(window_id).await?;
        }

        niri::set_width(window_id, total.saturating_sub(width)).await?;

        *self.narrowed.lock().await = Some(Narrowed {
            window_id,
            full_width: total,
            was_fullscreen: fullscreen,
        });

        Ok(())
    }

    /// The width goes back before the fullscreen does. A window that is fullscreen again ignores
    /// its column width, and would take the narrow one the next time it left.
    async fn give_room_back(&self, narrowed: &Narrowed) -> anyhow::Result<()> {
        niri::set_width(narrowed.window_id, narrowed.full_width).await?;

        if narrowed.was_fullscreen {
            niri::toggle_windowed_fullscreen(narrowed.window_id).await?;
        }

        Ok(())
    }
}
