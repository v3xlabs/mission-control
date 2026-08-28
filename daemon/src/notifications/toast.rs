use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::warn;

use crate::{niri, state::AppState};

use super::Surface;

const MARGIN: f64 = 24.0;

/// Holds the focus while it is up, because niri draws a floating window behind a fullscreen one
/// otherwise, and the display browser is usually fullscreen.
pub struct Toast {
    surface: Surface,
    /// The toast takes the focus to sit above a fullscreen display, so closing has to hand it
    /// back. Nothing else records that it was taken.
    took_focus: Mutex<bool>,
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
            took_focus: Mutex::new(false),
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

        if !std::mem::replace(&mut *self.took_focus.lock().await, false) {
            return;
        }

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

    /// The top right of the content, not of the output: those differ by the rail's width while it
    /// is open, and a toast anchored to the output would land on top of the agenda.
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

        *self.took_focus.lock().await = true;

        Ok(())
    }
}
