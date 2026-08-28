use std::{path::PathBuf, time::Duration};

use anyhow::{anyhow, Context as _, Result};
use tokio::{
    io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader},
    net::UnixStream,
    process::Child,
    sync::Mutex,
};

/// How long mpv gets to answer a command before the caller gives up on it.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// One mpv process, kept alive and idle between the files it is given.
///
/// A process launch on every switch would be visible on the wall, so mpv is started once and told
/// what to play over its control socket. Passing a file that way also keeps a stream url, which
/// holds a credential, out of the process list.
pub struct Mpv {
    process: Mutex<Option<Child>>,
    socket: PathBuf,
}

impl Mpv {
    pub fn new(socket: PathBuf) -> Self {
        Self {
            process: Mutex::new(None),
            socket,
        }
    }

    /// Starts mpv if it is not running, with the arguments that decide what kind of window it
    /// draws. `--idle` and `--force-window=no` are what let one process hold a window only while
    /// it has something to show.
    pub async fn ensure_running(&self, binary: &str, arguments: &[String]) -> Result<()> {
        {
            let mut process = self.process.lock().await;

            if let Some(running) = process.as_mut() {
                if matches!(running.try_wait(), Ok(None)) {
                    return Ok(());
                }
            }

            let _ = tokio::fs::remove_file(&self.socket).await;

            let mut command = tokio::process::Command::new(binary);

            command
                // A user's own mpv.conf would otherwise reach a wall display, where nobody is
                // sitting to undo what it does.
                .arg("--no-config")
                .arg("--no-terminal")
                .arg("--no-osc")
                .arg("--no-input-default-bindings")
                .arg("--idle=yes")
                .arg("--force-window=no")
                .arg(format!("--input-ipc-server={}", self.socket.display()));

            for argument in arguments {
                command.arg(argument);
            }

            *process = Some(command.spawn().context("cannot start mpv")?);
        }

        self.wait_for_socket().await
    }

    pub async fn command(&self, arguments: &[&str]) -> Result<()> {
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

    pub async fn shutdown(&self) {
        if let Some(mut process) = self.process.lock().await.take() {
            let _ = process.kill().await;
        }

        let _ = tokio::fs::remove_file(&self.socket).await;
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
        let dir = std::env::temp_dir().join(format!("missiond-test-mpv-{name}"));

        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir.join("mpv.sock")
    }

    #[tokio::test]
    async fn mpv_starts_idle_and_answers_on_its_socket() {
        let mpv = Mpv::new(scratch("idle"));

        mpv.ensure_running("mpv", &[]).await.unwrap();
        mpv.command(&["stop"]).await.unwrap();

        mpv.shutdown().await;

        assert!(mpv.command(&["stop"]).await.is_err());
    }

    #[tokio::test]
    async fn a_refused_command_is_an_error() {
        let mpv = Mpv::new(scratch("refused"));

        mpv.ensure_running("mpv", &[]).await.unwrap();

        let refused = mpv
            .command(&["set_property", "no-such-property", "1"])
            .await;

        mpv.shutdown().await;

        assert!(refused.is_err());
    }
}
