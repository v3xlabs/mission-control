use std::{sync::Arc, time::Duration};

use anyhow::Result;
use poem::{
    endpoint::EmbeddedFilesEndpoint,
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::Cors,
    web::{
        sse::{Event, SSE},
        Data, Path,
    },
    Body, EndpointExt as _, IntoResponse, Response, Route, Server,
};
use rust_embed::RustEmbed;
use tracing::info;

use crate::{api, state::AppState};

#[derive(RustEmbed)]
#[folder = "web-dist"]
struct WebAssets;

pub async fn start_http(state: Arc<AppState>) -> Result<()> {
    let http = state.config.read().await.device.http;
    let address = format!("{}:{}", http.host, http.port);

    info!(%address, "starting http server");

    let api_service = api::create_api_service(state.clone());
    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();

    let app = Route::new()
        .at("/api/preview/:tab_id", get(preview).data(state.clone()))
        .at(
            "/api/preview_live/:tab_id",
            get(preview_live).data(state.clone()),
        )
        .at("/api/screen", get(screen).data(state.clone()))
        .at("/api/events", get(events).data(state.clone()))
        .nest("/api", api_service)
        .nest("/docs", ui)
        .at("/docs/spec", spec)
        .nest("/", EmbeddedFilesEndpoint::<WebAssets>::new())
        .with(Cors::new());

    Server::new(TcpListener::bind(address)).run(app).await?;

    Ok(())
}

#[handler]
async fn preview(state: Data<&Arc<AppState>>, tab_id: Path<String>) -> impl IntoResponse {
    let Some(receiver) = state.chrome.watch_preview(&tab_id.0).await else {
        return not_found("no such tab");
    };

    let frame = receiver.borrow().clone();

    match frame {
        Some(frame) => Response::builder()
            .content_type("image/jpeg")
            .body(Body::from_bytes(frame.into())),
        None => not_found("no frame captured yet"),
    }
}

fn not_found(message: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(message.to_string())
}

/// What the compositor is putting on the panel, captured through its screencopy protocol rather
/// than through the browser. Unlike a tab preview this includes anything drawn over the page.
#[handler]
async fn screen(state: Data<&Arc<AppState>>) -> impl IntoResponse {
    let display = state.config.read().await.display;

    match state.capture.grab(&display).await {
        Ok(image) => Response::builder()
            .content_type("image/jpeg")
            .header("cache-control", "no-store")
            .body(Body::from_bytes(image.into())),
        Err(error) => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(error.to_string()),
    }
}

/// How long the stream waits for a new frame before resending the last one.
const HEARTBEAT: Duration = Duration::from_secs(2);

#[handler]
async fn preview_live(state: Data<&Arc<AppState>>, tab_id: Path<String>) -> impl IntoResponse {
    const BOUNDARY: &str = "missiondframe";

    let Some(mut receiver) = state.chrome.watch_preview(&tab_id.0).await else {
        return not_found("no such tab");
    };

    let stream = async_stream::stream! {
        yield Ok::<_, std::io::Error>(format!("--{BOUNDARY}\r\n").into_bytes());

        loop {
            let frame = receiver.borrow_and_update().clone();

            if let Some(frame) = frame {
                let part = format!(
                    "Content-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                    frame.len()
                );

                yield Ok::<_, std::io::Error>(part.into_bytes());
                yield Ok(frame);
                // The trailing boundary goes out with the frame rather than waiting for the next
                // one. A part is only complete once the following boundary arrives, so a tab that
                // produces one frame and then stops would otherwise never render at all.
                yield Ok(format!("\r\n--{BOUNDARY}\r\n").into_bytes());
            }

            // Chromium commits a part only once another one follows it, so a tab whose page has
            // stopped repainting needs the last frame sent again to appear at all.
            if let Ok(Err(_)) = tokio::time::timeout(HEARTBEAT, receiver.changed()).await {
                break;
            }
        }
    };

    Response::builder()
        .content_type(format!("multipart/x-mixed-replace; boundary={BOUNDARY}"))
        .body(Body::from_bytes_stream(stream))
}

#[handler]
fn events(state: Data<&Arc<AppState>>) -> SSE {
    let mut receiver = state.events.subscribe();

    SSE::new(async_stream::stream! {
        loop {
            let event = receiver.borrow_and_update().clone();

            if let Ok(payload) = serde_json::to_string(&event) {
                yield Event::message(payload);
            }

            if receiver.changed().await.is_err() {
                break;
            }
        }
    })
    .keep_alive(Duration::from_secs(30))
}
