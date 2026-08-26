pub mod mpv;
pub mod niri;
pub mod overlay;

use anyhow::{Context as _, Result};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::config::{MpvConfig, SecretRef};

use self::mpv::Mpv;

/// The app id the camera window carries, so the daemon can find it again and a compositor rule
/// can match it.
pub const APP_ID: &str = "missiond-camera";

/// The camera window.
///
/// A browser has no `rtsp://` handler, so a camera is played by mpv in its own window. The
/// compositor puts that window over the display and takes it away again, which is what lets a
/// camera behave like any other tab without the browser having to understand the stream.
pub struct Player {
    mpv: Mpv,
    showing: Mutex<Option<String>>,
}

impl Player {
    pub fn new(state: &std::path::Path) -> Self {
        Self {
            mpv: Mpv::new(state.join("mpv.sock")),
            showing: Mutex::new(None),
        }
    }

    pub async fn show(&self, tab_id: &str, stream: &SecretRef, config: &MpvConfig) -> Result<()> {
        self.mpv
            .ensure_running(&config.binary(), &arguments(config))
            .await?;

        let url = stream
            .resolve()
            .with_context(|| format!("cannot read the stream url for {tab_id}"))?;

        self.mpv.command(&["loadfile", &url]).await?;
        *self.showing.lock().await = Some(tab_id.to_string());

        info!("camera {tab_id} playing");

        Ok(())
    }

    /// Takes the window away, which is what reveals the page underneath.
    pub async fn hide(&self) {
        let Some(tab_id) = self.showing.lock().await.take() else {
            return;
        };

        match self.mpv.command(&["stop"]).await {
            Ok(()) => info!("camera {tab_id} stopped"),
            Err(error) => warn!("could not stop camera {tab_id}: {error}"),
        }
    }

    pub async fn shutdown(&self) {
        *self.showing.lock().await = None;

        self.mpv.shutdown().await;
    }
}

fn arguments(config: &MpvConfig) -> Vec<String> {
    let mut arguments = vec![
        "--profile=low-latency".to_string(),
        "--fullscreen".to_string(),
        // A camera that drops leaves its last frame up. Letting the window close would put
        // whatever page is behind it on the wall instead.
        "--keep-open=yes".to_string(),
        format!("--wayland-app-id={APP_ID}"),
    ];

    arguments.extend(config.extra_args.iter().cloned());
    arguments
}
