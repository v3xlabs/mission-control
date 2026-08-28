use std::path::Path;

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{config::MpvConfig, niri};

use super::mpv::Mpv;

/// The app id the clip window carries, so a compositor rule can match it.
const APP_ID: &str = "missiond-stinger";

/// A clip drawn over whatever is on screen.
///
/// The clip covers the seconds a camera needs to connect, so what is underneath has to show
/// through it. That rules out a page in the browser: the browser cannot draw over a window it
/// does not own, and a page has a background of its own where the stream should be. mpv draws the
/// clip in a window the compositor floats over the display instead.
pub struct Overlay {
    mpv: Mpv,
    playing: Mutex<bool>,
}

impl Overlay {
    pub fn new(state: &Path) -> Self {
        Self {
            mpv: Mpv::new(state.join("stinger.sock")),
            playing: Mutex::new(false),
        }
    }

    /// Puts the clip on screen and returns once it is up, so the caller can start whatever the
    /// clip is there to cover.
    pub async fn start(&self, file: &Path, config: &MpvConfig) -> Result<()> {
        self.mpv
            .ensure_running(&config.binary(), &arguments())
            .await?;

        self.mpv
            .command(&["loadfile", &file.to_string_lossy()])
            .await?;

        *self.playing.lock().await = true;

        niri::float(APP_ID).await?;

        info!("clip {} playing", file.display());

        Ok(())
    }

    /// Puts the clip back on top after something else took the focus. The camera has to be
    /// focused for the compositor to show its column at all, and that focus is what would
    /// otherwise leave the clip behind it.
    pub async fn raise(&self) {
        if !*self.playing.lock().await {
            return;
        }

        if let Err(error) = niri::float(APP_ID).await {
            warn!("could not put the clip back on top: {error}");
        }
    }

    /// Whether a clip is on screen. The window holds the focus while it is, which is what keeps
    /// it above a fullscreen camera.
    pub async fn is_playing(&self) -> bool {
        *self.playing.lock().await
    }

    /// Takes the clip away. The window goes with it, because mpv holds one only while it has
    /// something to play.
    pub async fn stop(&self) {
        if !std::mem::replace(&mut *self.playing.lock().await, false) {
            return;
        }

        if let Err(error) = self.mpv.command(&["stop"]).await {
            warn!("could not take the clip away: {error}");
        }
    }

    pub async fn shutdown(&self) {
        *self.playing.lock().await = false;

        self.mpv.shutdown().await;
    }
}

fn arguments() -> Vec<String> {
    vec![
        // Transparent where the clip is, so the display shows through it.
        "--background=none".to_string(),
        // VP9 keeps its alpha channel outside the frame and only libvpx's decoder reads it. mpv
        // still picks the right decoder for a clip that is not VP9.
        "--vd=libvpx-vp9".to_string(),
        // The window covers the output rather than taking the size of the file.
        "--autofit=100%x100%".to_string(),
        "--keep-open=no".to_string(),
        format!("--wayland-app-id={APP_ID}"),
    ]
}
