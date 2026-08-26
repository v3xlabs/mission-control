use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader},
    net::UnixStream,
};

/// mpv opens its window while `loadfile` has already been answered, so the first look at the
/// window list is usually too early.
const ATTEMPTS: usize = 30;
const POLL: Duration = Duration::from_millis(100);

#[derive(Deserialize)]
struct Window {
    id: u64,
    app_id: Option<String>,
}

/// Focuses the window carrying `app_id`.
///
/// niri holds every window in one scrolling layout, and it hands the focus to a new window only
/// when the client can show that a person opened it. A player started by a daemon cannot, so the
/// camera window arrives in a column beside the browser and stays off the panel until something
/// focuses it. niri answers json on the socket it names in `NIRI_SOCKET`, which is cheaper than a
/// shell script and needs nothing on the daemon's PATH.
pub async fn focus(app_id: &str) -> Result<()> {
    focus_on(&socket()?, app_id).await
}

/// Lifts the window carrying `app_id` out of the tiling layout and focuses it.
///
/// niri draws a floating window over the tiled ones, which is the only stacking it offers a
/// client that cannot speak layer-shell. The focus goes with it, because a fullscreen window
/// covers floating windows while it is the focused one, and the camera is fullscreen.
pub async fn float(app_id: &str) -> Result<()> {
    let socket = socket()?;
    let id = wait_for_window(&socket, app_id).await?;

    request(&socket, &json!({ "Action": { "MoveWindowToFloating": { "id": id } } })).await?;
    request(&socket, &json!({ "Action": { "FocusWindow": { "id": id } } })).await?;

    Ok(())
}

fn socket() -> Result<PathBuf> {
    std::env::var("NIRI_SOCKET")
        .map(PathBuf::from)
        .context("NIRI_SOCKET is not set")
}

async fn focus_on(socket: &Path, app_id: &str) -> Result<()> {
    let id = wait_for_window(socket, app_id).await?;

    request(socket, &json!({ "Action": { "FocusWindow": { "id": id } } })).await?;

    Ok(())
}

async fn wait_for_window(socket: &Path, app_id: &str) -> Result<u64> {
    for _ in 0..ATTEMPTS {
        if let Some(id) = window_id(socket, app_id).await? {
            return Ok(id);
        }

        tokio::time::sleep(POLL).await;
    }

    Err(anyhow!("no {app_id} window appeared"))
}

async fn window_id(socket: &Path, app_id: &str) -> Result<Option<u64>> {
    let windows = request(socket, &json!("Windows")).await?;
    let windows: Vec<Window> = serde_json::from_value(
        windows
            .get("Windows")
            .cloned()
            .ok_or_else(|| anyhow!("niri answered a window list request with something else"))?,
    )?;

    Ok(windows
        .into_iter()
        .find(|window| window.app_id.as_deref() == Some(app_id))
        .map(|window| window.id))
}

/// One request, one reply, one connection: niri closes the socket once it has answered.
async fn request(socket: &Path, payload: &Value) -> Result<Value> {
    let mut stream = UnixStream::connect(socket)
        .await
        .context("niri is not listening")?;

    stream.write_all(payload.to_string().as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut line = String::new();

    BufReader::new(stream)
        .read_line(&mut line)
        .await
        .context("niri closed the socket without answering")?;

    let reply: Value = serde_json::from_str(&line).context("niri answered with something else")?;

    match reply.get("Err") {
        Some(error) => Err(anyhow!("niri refused the request: {error}")),
        None => reply
            .get("Ok")
            .cloned()
            .ok_or_else(|| anyhow!("niri answered without a result")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokio::net::UnixListener;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("missiond-test-niri-{name}"));

        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir.join("niri.sock")
    }

    /// Reads the request off one connection and answers it, the way niri does.
    async fn exchange(listener: &UnixListener, reply: &Value) -> Value {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        reader.read_line(&mut line).await.unwrap();

        let mut stream = reader.into_inner();

        stream.write_all(reply.to_string().as_bytes()).await.unwrap();
        stream.write_all(b"\n").await.unwrap();

        serde_json::from_str(&line).unwrap()
    }

    #[tokio::test]
    async fn the_window_holding_the_app_id_is_the_one_focused() {
        let socket = scratch("focus");
        let listener = UnixListener::bind(&socket).unwrap();

        let niri = tokio::spawn(async move {
            let listed = exchange(
                &listener,
                &json!({"Ok": {"Windows": [
                    {"id": 2, "app_id": "chromium-browser"},
                    {"id": 7, "app_id": "missiond-camera"}
                ]}}),
            )
            .await;
            let acted = exchange(&listener, &json!({"Ok": "Handled"})).await;

            (listed, acted)
        });

        focus_on(&socket, "missiond-camera").await.unwrap();

        let (listed, acted) = niri.await.unwrap();

        assert_eq!(listed, json!("Windows"));
        assert_eq!(acted, json!({"Action": {"FocusWindow": {"id": 7}}}));
    }

    #[tokio::test]
    async fn a_refusal_is_an_error() {
        let socket = scratch("refused");
        let listener = UnixListener::bind(&socket).unwrap();

        tokio::spawn(async move {
            exchange(&listener, &json!({"Err": "unknown request"})).await;
        });

        assert!(focus_on(&socket, "missiond-camera").await.is_err());
    }
}
