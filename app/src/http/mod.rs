use std::{sync::Arc, time::Duration};

use tracing::info;
use anyhow::Result;
use base64::Engine;
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::Cors,
    web::{Data, Path},
    Body, EndpointExt as _, IntoResponse, Response, Route, Server,
};
use poem_openapi::OpenApiService;

use crate::{api, state::AppState};

pub async fn start_http(state: Arc<AppState>) -> Result<()> {
    info!("Starting HTTP server on port 3000");

    // Create OpenAPI service and Swagger UI
    let api_service: OpenApiService<api::ManagementApi, ()> = api::create_api_service(state.clone());
    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();

    let app = Route::new()
        .at("/api/preview/:tab_id", get(preview).data(state.clone()))
        .at("/api/preview_live/:tab_id", get(preview_live).data(state.clone()))
        .nest("/api", api_service)
        .nest("/docs", ui)
        .at("/docs/spec", spec)
        .with(Cors::new());

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
    info!("preview: {}", tab_id.0);

    let last_frames = state.chrome.last_frame.lock().await;
    let body = last_frames.get(&tab_id.0).unwrap();

    // base64 decode body
    let body = base64::engine::general_purpose::STANDARD
        .decode(body)
        .unwrap();

    info!("body: {:?}", body.len());

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
            async_std::task::sleep(Duration::from_millis(1000 / 4)).await;
        }
    };

    // Build a response with the appropriate multipart content type.
    Response::builder()
        .body(Body::from_bytes_stream(stream))
        .set_content_type(format!("multipart/x-mixed-replace; boundary={}", boundary))
        .into_response()
}
