use std::sync::Arc;

use anyhow::{Context as _, Result};
use tokio::{process::Child, sync::Mutex};
use tracing::{info, warn};

use crate::{chrome::mark_clean_exit, niri, state::AppState};

/// A page the daemon puts on the compositor in a window of its own.
///
/// This is a second browser rather than a layer-shell surface, because the compositor is already a
/// tiling one and speaks an IPC the daemon can drive. The window uses Chromium's app mode, so it
/// carries no tab strip and no address bar, and its own profile, so it cannot disturb the
/// display's.
pub struct Surface {
    /// A directory under the cache directory, so two surfaces never share a profile lock.
    profile: &'static str,
    /// The page the window opens, served by the daemon itself.
    page: &'static str,
    window: Mutex<Option<Child>>,
}

impl Surface {
    pub fn new(profile: &'static str, page: &'static str) -> Self {
        Self {
            profile,
            page,
            window: Mutex::new(None),
        }
    }

    /// Whether the window is up.
    ///
    /// A `Child` that has exited is still `Some`, so the handle alone cannot answer this. Reading
    /// it as if it could is what leaves a surface that crashed at nine in the morning closed for
    /// the rest of the day.
    pub async fn is_running(&self) -> bool {
        let mut window = self.window.lock().await;

        let Some(child) = window.as_mut() else {
            return false;
        };

        match child.try_wait() {
            Ok(None) => true,
            Ok(Some(status)) => {
                warn!(self.page, %status, "surface window exited");
                *window = None;

                false
            }
            Err(error) => {
                warn!(self.page, "cannot tell whether the surface window is alive: {error}");

                false
            }
        }
    }

    /// Opens the window and returns the compositor's id for it, so the caller can place it.
    ///
    /// The window is found by the process that owns it. Chromium derives the app id of an `--app`
    /// window from its URL and ignores `--class`, so a name the daemon chose is not something the
    /// compositor ever sees.
    pub async fn open(&self, app_state: &Arc<AppState>) -> Result<u64> {
        let config = app_state.config.read().await;
        let port = config.device.http.port;
        let binary = config
            .device
            .chromium
            .binary_path
            .clone()
            .or_else(|| std::env::var("CHROMIUM_BINARY").ok())
            .unwrap_or_else(|| "chromium".to_string());

        let profile = app_state.config.dirs.cache.join(self.profile);
        let page = self.page;

        // A surface is killed rather than closed, every time. Without this the next one comes up
        // asking whether to restore its pages, over the alert it was opened to show.
        mark_clean_exit(&profile);

        let child = tokio::process::Command::new(binary)
            .arg(format!("--app=http://127.0.0.1:{port}/{page}"))
            .arg(format!("--user-data-dir={}", profile.display()))
            .arg("--ozone-platform=wayland")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--password-store=basic")
            .arg("--use-mock-keychain")
            .arg("--disable-extensions")
            .arg("--disable-sync")
            // A surface shows one local page. Without this it still tries to register for push
            // notifications on every open, and fills the log with the failures.
            .arg("--disable-background-networking")
            // A display that is killed rather than closed comes back asking whether to restore its
            // pages. Nobody is standing at the wall to answer, and the dialog sits over the alert.
            .arg("--disable-session-crashed-bubble")
            .arg("--hide-crash-restore-bubble")
            .spawn()
            .with_context(|| format!("cannot open the window for {page}"))?;

        let pid = child.id().context("the surface window reported no process id")?;

        *self.window.lock().await = Some(child);

        let window_id = niri::wait_for_pid(pid).await?.id;

        info!(page, window_id, "surface window opened");

        Ok(window_id)
    }

    pub async fn close(&self) {
        if let Some(mut child) = self.window.lock().await.take() {
            let _ = child.kill().await;
            info!(self.page, "surface window closed");
        }
    }
}
