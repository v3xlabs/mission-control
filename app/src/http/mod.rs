use std::{convert::Infallible, ops::Deref, str::Bytes, sync::Arc, time::Duration};

use anyhow::Result;
use async_std::task;
use base64::Engine;
use poem::{
    get, handler,
    listener::TcpListener,
    web::{Data, Path},
    Body, EndpointExt as _, IntoResponse, Response, Route, Server,
};

use crate::state::AppState;

pub async fn start_http(state: Arc<AppState>) -> Result<()> {
    let app = Route::new()
        .at("/", get(root))
        .at("/preview/:tab_id", get(preview))
        .at("/preview_live/:tab_id", get(preview_live))
        .data(state);

    let server = Server::new(TcpListener::bind("0.0.0.0:3000"));
    server.run(app).await?;

    Ok(())
}

#[handler]
async fn root() -> &'static str {
    "v3x-mission-control"
}

#[handler]
async fn preview(state: Data<&Arc<AppState>>, tab_id: Path<String>) -> impl IntoResponse {
    format!("preview: {}", tab_id.0);

    let last_frames = state.chrome.last_frame.lock().await;
    let body = last_frames.get(&tab_id.0).unwrap();

    // base64 decode body
    let body = base64::engine::general_purpose::STANDARD
        .decode(body)
        .unwrap();

    println!("body: {:?}", body.len());

    Response::builder()
        .body(Body::from_bytes(body.clone().into()))
        .set_content_type("image/jpeg")
        .into_response()
}

#[handler]
async fn preview_live(state: Data<&Arc<AppState>>, tab_id: Path<String>) -> impl IntoResponse {
    let boundary = "myboundary";
    let xstate = state.clone();
    
    // Create a stream that loops indefinitely, yielding a new frame each iteration.
    let stream = async_stream::stream! {
        loop {
            // Lock the shared state and fetch the latest frame for the given tab.
            let last_frames = xstate.chrome.last_frame.lock().await;
            if let Some(encoded) = last_frames.get(&tab_id.0) {
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                    let header = format!(
                        "--{}\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        boundary,
                        decoded.len()
                    );
                    // Yield the header, the JPEG bytes, and a CRLF.
                    yield Ok::<_, std::io::Error>(header.into_bytes());
                    yield Ok(decoded);
                    yield Ok(b"\r\n".to_vec());
                }
            }
            // Wait a short duration before sending the next frame.
            async_std::task::sleep(Duration::from_millis(100)).await;
        }
    };

    // Build a response with the appropriate multipart content type.
    Response::builder()
        .body(Body::from_bytes_stream(stream))
        .set_content_type(format!("multipart/x-mixed-replace; boundary={}", boundary))
        .into_response()
}
