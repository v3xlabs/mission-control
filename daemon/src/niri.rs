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
use tracing::debug;

/// A window appears in the list some time after the call that asked for it has been answered.
const ATTEMPTS: usize = 30;
const POLL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Deserialize)]
pub struct Window {
    pub id: u64,
    pub app_id: Option<String>,
    pub pid: Option<i32>,
    pub layout: Layout,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Layout {
    /// Logical pixels.
    pub window_size: [u32; 2],
}

/// niri hands the focus to a new window only when the client can show that a person opened it. A
/// window the daemon started cannot, so it stays off the panel until something focuses it.
pub async fn focus(app_id: &str) -> Result<()> {
    let socket = socket()?;
    let window = wait_for(&socket, &by_app_id(app_id)).await?;

    focus_window(&socket, window.id).await
}

/// Floating is the only stacking niri offers a client that cannot speak layer-shell. The focus
/// goes with it, because a focused fullscreen window covers floating ones.
pub async fn float(app_id: &str) -> Result<()> {
    let socket = socket()?;
    let id = wait_for(&socket, &by_app_id(app_id)).await?.id;

    request(
        &socket,
        &json!({ "Action": { "MoveWindowToFloating": { "id": id } } }),
    )
    .await?;
    focus_window(&socket, id).await
}

/// Chromium derives the app id of an `--app=` window from its URL and ignores `--class`, so a
/// window opened for a surface cannot be found by a name the daemon chose.
pub async fn wait_for_pid(pid: u32) -> Result<Window> {
    let socket = socket()?;

    wait_for(&socket, &|window: &Window| window.pid == Some(pid as i32)).await
}

/// Logical pixels. A tiling compositor sizes the column itself, so `--window-size` on the client
/// is a request nothing reads.
pub async fn set_width(id: u64, pixels: u32) -> Result<()> {
    request(
        &socket()?,
        &json!({ "Action": { "SetWindowWidth": { "id": id, "change": { "SetFixed": pixels } } } }),
    )
    .await?;

    Ok(())
}

/// Logical pixels. Only a floating window has a height of its own.
pub async fn set_height(id: u64, pixels: u32) -> Result<()> {
    request(
        &socket()?,
        &json!({ "Action": { "SetWindowHeight": { "id": id, "change": { "SetFixed": pixels } } } }),
    )
    .await?;

    Ok(())
}

/// Fullscreen as far as the client is concerned, while the compositor keeps the window in the
/// tiling layout: Chromium hides its tab strip and its omnibox only when it believes it is
/// fullscreen, and a really fullscreen window covers the output rather than sharing it.
///
/// niri offers a toggle rather than a setter, so the caller has to know the state it is leaving.
pub async fn toggle_windowed_fullscreen(id: u64) -> Result<()> {
    request(
        &socket()?,
        &json!({ "Action": { "ToggleWindowedFullscreen": { "id": id } } }),
    )
    .await?;

    Ok(())
}

pub async fn place_floating(id: u64, x: f64, y: f64) -> Result<()> {
    let socket = socket()?;

    request(
        &socket,
        &json!({ "Action": { "MoveWindowToFloating": { "id": id } } }),
    )
    .await?;
    request(
        &socket,
        &json!({ "Action": { "MoveFloatingWindow": {
            "id": id,
            "x": { "SetFixed": x },
            "y": { "SetFixed": y },
        } } }),
    )
    .await?;

    Ok(())
}

pub async fn focus_id(id: u64) -> Result<()> {
    focus_window(&socket()?, id).await
}

/// `logical` rather than the mode, because a scaled output reports its mode in physical pixels and
/// a column asked for in those is twice the width it should be.
pub async fn output_width(name: Option<&str>) -> Result<u32> {
    #[derive(Deserialize)]
    struct Logical {
        width: u32,
    }

    #[derive(Deserialize)]
    struct Output {
        logical: Option<Logical>,
    }

    let reply = request(&socket()?, &json!("Outputs")).await?;
    let outputs: std::collections::HashMap<String, Output> = serde_json::from_value(
        reply
            .get("Outputs")
            .cloned()
            .ok_or_else(|| anyhow!("niri answered an output list request with something else"))?,
    )?;

    match name {
        Some(name) => outputs
            .get(name)
            .and_then(|output| output.logical.as_ref())
            .map(|logical| logical.width)
            .ok_or_else(|| anyhow!("niri reported no enabled output called {name}")),
        None => outputs
            .into_values()
            .find_map(|output| Some(output.logical?.width))
            .ok_or_else(|| anyhow!("niri reported no enabled output")),
    }
}

fn by_app_id(app_id: &str) -> impl Fn(&Window) -> bool + '_ {
    move |window: &Window| window.app_id.as_deref() == Some(app_id)
}

fn socket() -> Result<PathBuf> {
    std::env::var("NIRI_SOCKET")
        .map(PathBuf::from)
        .context("NIRI_SOCKET is not set")
}

async fn focus_window(socket: &Path, id: u64) -> Result<()> {
    request(
        socket,
        &json!({ "Action": { "FocusWindow": { "id": id } } }),
    )
    .await?;

    Ok(())
}

async fn wait_for(socket: &Path, matches: &impl Fn(&Window) -> bool) -> Result<Window> {
    for _ in 0..ATTEMPTS {
        if let Some(window) = find(socket, matches).await? {
            return Ok(window);
        }

        tokio::time::sleep(POLL).await;
    }

    Err(anyhow!("no window appeared"))
}

async fn find(socket: &Path, matches: &impl Fn(&Window) -> bool) -> Result<Option<Window>> {
    let reply = request(socket, &json!("Windows")).await?;
    let windows: Vec<Window> = serde_json::from_value(
        reply
            .get("Windows")
            .cloned()
            .ok_or_else(|| anyhow!("niri answered a window list request with something else"))?,
    )?;

    Ok(windows.into_iter().find(matches))
}

/// One request, one reply, one connection: niri closes the socket once it has answered.
/// Retried once. niri has been seen accepting a connection and closing it without answering while
/// several windows are opening and closing at once. Losing that request leaves the display
/// narrowed with nothing beside it, which is a wrong screen rather than a missing log line.
async fn request(socket: &Path, payload: &Value) -> Result<Value> {
    match request_once(socket, payload).await {
        Ok(reply) => Ok(reply),
        Err(first) => {
            debug!("retrying a niri request: {first}");

            request_once(socket, payload).await
        }
    }
}

async fn request_once(socket: &Path, payload: &Value) -> Result<Value> {
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

    async fn exchange(listener: &UnixListener, reply: &Value) -> Value {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        reader.read_line(&mut line).await.unwrap();

        let mut stream = reader.into_inner();

        stream
            .write_all(reply.to_string().as_bytes())
            .await
            .unwrap();
        stream.write_all(b"\n").await.unwrap();

        serde_json::from_str(&line).unwrap()
    }

    fn windows() -> Value {
        json!({"Ok": {"Windows": [
            {"id": 2, "app_id": "chromium-browser", "pid": 100,
             "layout": {"window_size": [1280, 800]}},
            {"id": 7, "app_id": "missiond-camera", "pid": 200,
             "layout": {"window_size": [640, 800]}}
        ]}})
    }

    #[tokio::test]
    async fn the_window_holding_the_app_id_is_the_one_focused() {
        let socket = scratch("focus");
        let listener = UnixListener::bind(&socket).unwrap();

        let niri = tokio::spawn(async move {
            let listed = exchange(&listener, &windows()).await;
            let acted = exchange(&listener, &json!({"Ok": "Handled"})).await;

            (listed, acted)
        });

        let window = wait_for(&socket, &by_app_id("missiond-camera"))
            .await
            .unwrap();

        focus_window(&socket, window.id).await.unwrap();

        let (listed, acted) = niri.await.unwrap();

        assert_eq!(listed, json!("Windows"));
        assert_eq!(acted, json!({"Action": {"FocusWindow": {"id": 7}}}));
    }

    #[tokio::test]
    async fn a_window_is_found_by_the_process_that_owns_it() {
        let socket = scratch("pid");
        let listener = UnixListener::bind(&socket).unwrap();

        tokio::spawn(async move {
            exchange(&listener, &windows()).await;
        });

        let window = wait_for(&socket, &|window: &Window| window.pid == Some(200))
            .await
            .unwrap();

        assert_eq!(window.id, 7);
    }

    #[tokio::test]
    async fn a_refusal_is_an_error() {
        let socket = scratch("refused");
        let listener = UnixListener::bind(&socket).unwrap();

        tokio::spawn(async move {
            exchange(&listener, &json!({"Err": "unknown request"})).await;
        });

        assert!(wait_for(&socket, &by_app_id("missiond-camera"))
            .await
            .is_err());
    }
}
