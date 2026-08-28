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

/// A window opens while the call that asked for it has already been answered, so the first look at
/// the window list is usually too early.
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
    /// Width and height in logical pixels, which is what a window actually occupies rather than
    /// what it was asked for.
    pub window_size: [u32; 2],
}

/// Focuses the window carrying `app_id`.
///
/// niri holds every window in one scrolling layout, and it hands the focus to a new window only
/// when the client can show that a person opened it. A player started by a daemon cannot, so the
/// camera window arrives in a column beside the browser and stays off the panel until something
/// focuses it. niri answers json on the socket it names in `NIRI_SOCKET`, which is cheaper than a
/// shell script and needs nothing on the daemon's PATH.
pub async fn focus(app_id: &str) -> Result<()> {
    let socket = socket()?;
    let window = wait_for(&socket, &by_app_id(app_id)).await?;

    focus_window(&socket, window.id).await
}

/// Lifts the window carrying `app_id` out of the tiling layout and focuses it.
///
/// niri draws a floating window over the tiled ones, which is the only stacking it offers a
/// client that cannot speak layer-shell. The focus goes with it, because a fullscreen window
/// covers floating windows while it is the focused one, and the camera is fullscreen.
pub async fn float(app_id: &str) -> Result<()> {
    let socket = socket()?;
    let id = wait_for(&socket, &by_app_id(app_id)).await?.id;

    request(&socket, &json!({ "Action": { "MoveWindowToFloating": { "id": id } } })).await?;
    focus_window(&socket, id).await
}

/// The window belonging to a process the daemon started.
///
/// Chromium derives the app id of an `--app=` window from its URL and ignores `--class`, so a
/// window opened for a surface cannot be found by a name the daemon chose. The process id can,
/// and the daemon owns the child either way.
pub async fn wait_for_pid(pid: u32) -> Result<Window> {
    let socket = socket()?;

    wait_for(&socket, &|window: &Window| window.pid == Some(pid as i32)).await
}

/// Sets a window's width in logical pixels.
///
/// This is what makes a configured sidebar width mean something. A tiling compositor sizes the
/// column itself, so `--window-size` on the client is a request nothing reads.
pub async fn set_width(id: u64, pixels: u32) -> Result<()> {
    request(
        &socket()?,
        &json!({ "Action": { "SetWindowWidth": { "id": id, "change": { "SetFixed": pixels } } } }),
    )
    .await?;

    Ok(())
}

/// Sets a window's height in logical pixels. Only a floating window has a height of its own.
pub async fn set_height(id: u64, pixels: u32) -> Result<()> {
    request(
        &socket()?,
        &json!({ "Action": { "SetWindowHeight": { "id": id, "change": { "SetFixed": pixels } } } }),
    )
    .await?;

    Ok(())
}

/// Makes a window fullscreen as far as the client is concerned while the compositor keeps it in
/// the tiling layout.
///
/// Chromium hides its tab strip and its omnibox only when it believes it is fullscreen, and a
/// really fullscreen window covers the output rather than sharing it. Windowed fullscreen is what
/// lets the display keep a browser with no interface and still make room for a column beside it.
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

/// Takes a window out of the tiling layout and puts it at a position on the output.
pub async fn place_floating(id: u64, x: f64, y: f64) -> Result<()> {
    let socket = socket()?;

    request(&socket, &json!({ "Action": { "MoveWindowToFloating": { "id": id } } })).await?;
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

/// Focuses a window the caller has already found.
pub async fn focus_id(id: u64) -> Result<()> {
    focus_window(&socket()?, id).await
}

/// The width of an output in logical pixels, which is the unit every window action takes.
///
/// `logical` rather than the mode, because a scaled output reports a mode in physical pixels and
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

    // Without a configured output name there is nothing to choose by, and a display with one
    // screen is the case this serves.
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
    request(socket, &json!({ "Action": { "FocusWindow": { "id": id } } })).await?;

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

        let window = wait_for(&socket, &by_app_id("missiond-camera")).await.unwrap();

        focus_window(&socket, window.id).await.unwrap();

        let (listed, acted) = niri.await.unwrap();

        assert_eq!(listed, json!("Windows"));
        assert_eq!(acted, json!({"Action": {"FocusWindow": {"id": 7}}}));
    }

    /// Chromium ignores `--class` on an `--app` window and derives an app id from the URL, so the
    /// process is the only handle the daemon has on a surface it opened.
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

        assert!(wait_for(&socket, &by_app_id("missiond-camera")).await.is_err());
    }
}
