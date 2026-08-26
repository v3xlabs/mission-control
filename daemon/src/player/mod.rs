use std::{path::PathBuf, time::Duration};

use anyhow::{anyhow, Context as _, Result};
use tokio::{
    io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader},
    net::UnixStream,
    process::Child,
    sync::Mutex,
};
use tracing::{info, warn};

use crate::config::{MpvConfig, SecretRef};

/// The app id the camera window carries, so a compositor rule can match it.
const APP_ID: &str = "missiond-camera";

/// How long mpv gets to answer a command before the caller gives up on it.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// The camera window.
///
/// A browser has no `rtsp://` handler, so a camera is played by mpv in its own window. The
/// compositor puts that window over the display and takes it away again, which is what lets a
/// camera behave like any other tab without the browser having to understand the stream.
///
/// mpv stays alive and idle between cameras rather than being started per stream. A process
/// launch on every switch would be visible on the wall, and the stream URL holds a credential:
/// handing it over the control socket keeps it out of the process list, which an argument could
/// not.
pub struct Player {
    process: Mutex<Option<Child>>,
    showing: Mutex<Option<String>>,
    socket: PathBuf,
}

impl Player {
    pub fn new(state: &std::path::Path) -> Self {
        Self {
            process: Mutex::new(None),
            showing: Mutex::new(None),
            socket: state.join("mpv.sock"),
        }
    }

    pub async fn show(&self, tab_id: &str, stream: &SecretRef, config: &MpvConfig) -> Result<()> {
        self.ensure_running(config).await?;

        let url = stream
            .resolve()
            .with_context(|| format!("cannot read the stream url for {tab_id}"))?;

        self.command(&["loadfile", &url]).await?;
        *self.showing.lock().await = Some(tab_id.to_string());

        info!("camera {tab_id} playing");

        Ok(())
    }

    /// Takes the window away, which is what reveals the page underneath.
    pub async fn hide(&self) {
        let Some(tab_id) = self.showing.lock().await.take() else {
            return;
        };

        match self.command(&["stop"]).await {
            Ok(()) => info!("camera {tab_id} stopped"),
            Err(error) => warn!("could not stop camera {tab_id}: {error}"),
        }
    }

    pub async fn shutdown(&self) {
        *self.showing.lock().await = None;

        if let Some(mut process) = self.process.lock().await.take() {
            let _ = process.kill().await;
        }

        let _ = tokio::fs::remove_file(&self.socket).await;
    }

    async fn ensure_running(&self, config: &MpvConfig) -> Result<()> {
        {
            let mut process = self.process.lock().await;

            if let Some(running) = process.as_mut() {
                if matches!(running.try_wait(), Ok(None)) {
                    return Ok(());
                }
            }

            let _ = tokio::fs::remove_file(&self.socket).await;

            let mut command = tokio::process::Command::new(config.binary());

            command
                // A user's own mpv.conf would otherwise reach a wall display, where nobody is
                // sitting to undo what it does.
                .arg("--no-config")
                .arg("--no-terminal")
                .arg("--no-osc")
                .arg("--no-input-default-bindings")
                .arg("--profile=low-latency")
                .arg("--idle=yes")
                .arg("--force-window=no")
                .arg("--fullscreen")
                // A camera that drops leaves its last frame up. Letting the window close would
                // put whatever page is behind it on the wall instead.
                .arg("--keep-open=yes")
                .arg(format!("--wayland-app-id={APP_ID}"))
                .arg(format!("--input-ipc-server={}", self.socket.display()));

            for argument in &config.extra_args {
                command.arg(argument);
            }

            *process = Some(command.spawn().context("cannot start mpv")?);
        }

        self.wait_for_socket().await
    }

    /// mpv creates the socket some way into its own startup, so the first command has to wait for
    /// it to appear.
    async fn wait_for_socket(&self) -> Result<()> {
        for _ in 0..50 {
            if UnixStream::connect(&self.socket).await.is_ok() {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(anyhow!("mpv did not open its control socket"))
    }

    async fn command(&self, arguments: &[&str]) -> Result<()> {
        let payload = serde_json::json!({ "command": arguments }).to_string();
        let mut stream = UnixStream::connect(&self.socket)
            .await
            .context("mpv is not listening")?;

        stream.write_all(payload.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let name = arguments[0];

        tokio::time::timeout(REPLY_TIMEOUT, read_reply(stream))
            .await
            .map_err(|_| anyhow!("mpv did not answer {name}"))?
            .map_err(|error| anyhow!("mpv refused {name}: {error}"))
    }
}

/// mpv sends events down the same socket as replies, so the first line is not necessarily the
/// answer. The reply is the one carrying an `error` field.
async fn read_reply(stream: UnixStream) -> Result<(), String> {
    let mut lines = BufReader::new(stream).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let Ok(message) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };

        let Some(error) = message.get("error").and_then(serde_json::Value::as_str) else {
            continue;
        };

        return if error == "success" {
            Ok(())
        } else {
            Err(error.to_string())
        };
    }

    Err("the control socket closed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("missiond-test-player-{name}"));

        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[tokio::test]
    async fn mpv_starts_idle_and_answers_on_its_socket() {
        let player = Player::new(&scratch("idle"));

        player.ensure_running(&MpvConfig::default()).await.unwrap();
        player.command(&["stop"]).await.unwrap();

        player.shutdown().await;

        assert!(player.command(&["stop"]).await.is_err());
    }

    #[tokio::test]
    async fn a_refused_command_is_an_error() {
        let player = Player::new(&scratch("refused"));

        player.ensure_running(&MpvConfig::default()).await.unwrap();

        let refused = player.command(&["set_property", "no-such-property", "1"]).await;

        player.shutdown().await;

        assert!(refused.is_err());
    }
}
