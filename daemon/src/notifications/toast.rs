use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::warn;

use crate::{niri, state::AppState};

use super::Surface;

/// How far the toast sits from the edges of the output.
const MARGIN: f64 = 24.0;

/// A small window over a corner of the content.
///
/// A meeting five minutes away is worth saying and not worth interrupting for. The toast floats,
/// so nothing is resized and the playlist keeps running underneath it. It holds the focus while it
/// is up, because a floating window sits behind a fullscreen one otherwise, and the display
/// browser is usually fullscreen.
pub struct Toast {
    surface: Surface,
    shown_over: Mutex<Option<u64>>,
}

impl Default for Toast {
    fn default() -> Self {
        Self::new()
    }
}

impl Toast {
    pub fn new() -> Self {
        Self {
            surface: Surface::new("toast-profile", "notify.html?toast=1"),
            shown_over: Mutex::new(None),
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
        let width = config.notifications.toast_width;
        let height = config.notifications.toast_height;

        let window_id = match self.surface.open(app_state).await {
            Ok(window_id) => window_id,
            Err(error) => {
                warn!("could not open the toast: {error}");

                return;
            }
        };

        if let Err(error) = self.place(app_state, window_id, width, height).await {
            warn!("could not place the toast: {error}");
        }
    }

    pub async fn close(&self, app_state: &Arc<AppState>) {
        self.surface.close().await;

        // The focus goes back to the content, or the display is left showing a window that is no
        // longer there in front of one that is.
        let Some(_) = self.shown_over.lock().await.take() else {
            return;
        };

        if let Some(pid) = app_state.chrome.browser_pid().await {
            match niri::wait_for_pid(pid).await {
                Ok(browser) => {
                    if let Err(error) = niri::focus_id(browser.id).await {
                        warn!("could not give the display back the focus: {error}");
                    }
                }
                Err(error) => warn!("could not find the display window: {error}"),
            }
        }
    }

    /// The toast sits at the top right of the content, not of the output.
    ///
    /// Those are the same thing until the rail is open, and then they are 480 pixels apart: a
    /// toast anchored to the output would land on top of the agenda it is announcing.
    async fn place(
        &self,
        app_state: &Arc<AppState>,
        window_id: u64,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let content = match app_state.chrome.browser_pid().await {
            Some(pid) => niri::wait_for_pid(pid).await?.layout.window_size[0],
            None => {
                let output = app_state.config.read().await.display.output;

                niri::output_width(output.as_deref()).await?
            }
        };

        let x = f64::from(content.saturating_sub(width)) - MARGIN;

        niri::place_floating(window_id, x.max(0.0), MARGIN).await?;
        niri::set_width(window_id, width).await?;
        niri::set_height(window_id, height).await?;
        niri::focus_id(window_id).await?;

        *self.shown_over.lock().await = Some(window_id);

        Ok(())
    }
}
