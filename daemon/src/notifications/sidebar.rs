use std::{sync::Arc, time::Duration};

use tokio::{process::Child, sync::Mutex};
use tracing::{info, warn};

use crate::state::AppState;

/// An alert shown beside the content rather than over it.
///
/// This is a second browser window rather than a layer-shell surface, because the compositor is
/// already a tiling one: it opens, niri gives it a column, and the page shrinks to make room. The
/// window uses Chromium's app mode, so it carries no tab strip or address bar, and its own
/// profile, so it cannot disturb the display's.
pub struct Sidebar {
    window: Mutex<Option<Child>>,
    expiry: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            window: Mutex::new(None),
            expiry: Mutex::new(None),
        }
    }

    pub async fn show(&self, app_state: &Arc<AppState>, lifetime: Duration) {
        let config = app_state.config.read().await;

        // A window already up shows the new alert too, because the page reads the whole list.
        if self.window.lock().await.is_some() {
            self.schedule_hide(app_state, lifetime).await;

            return;
        }

        let port = config.device.http.port;
        let width = config.notifications.sidebar_width;
        let binary = config
            .device
            .chromium
            .binary_path
            .clone()
            .or_else(|| std::env::var("CHROMIUM_BINARY").ok())
            .unwrap_or_else(|| "chromium".to_string());

        let profile = app_state.config.dirs.cache.join("sidebar-profile");

        let child = tokio::process::Command::new(binary)
            .arg(format!("--app=http://127.0.0.1:{port}/notify.html?sidebar=1"))
            .arg(format!("--user-data-dir={}", profile.display()))
            .arg(format!("--window-size={width},1080"))
            // Its own app id, so a window rule can place or size it without matching the display.
            .arg("--class=missiond-sidebar")
            .arg("--ozone-platform=wayland")
            .arg("--no-first-run")
            .arg("--password-store=basic")
            .arg("--use-mock-keychain")
            .arg("--disable-extensions")
            .arg("--disable-sync")
            .spawn();

        match child {
            Ok(child) => {
                info!("sidebar window opened");
                *self.window.lock().await = Some(child);
            }
            Err(error) => warn!("could not open the sidebar window: {error}"),
        }

        self.schedule_hide(app_state, lifetime).await;
    }

    pub async fn hide(&self) {
        if let Some(task) = self.expiry.lock().await.take() {
            task.abort();
        }

        if let Some(mut child) = self.window.lock().await.take() {
            let _ = child.kill().await;
            info!("sidebar window closed");
        }
    }

    /// A later alert pushes the close back, so a burst does not leave the window closing on the
    /// first one's schedule.
    async fn schedule_hide(&self, app_state: &Arc<AppState>, lifetime: Duration) {
        if let Some(task) = self.expiry.lock().await.take() {
            task.abort();
        }

        let state = app_state.clone();

        *self.expiry.lock().await = Some(tokio::spawn(async move {
            tokio::time::sleep(lifetime).await;

            if state.notifications.current().await.is_none() {
                state.sidebar.hide().await;
            }
        }));
    }
}
