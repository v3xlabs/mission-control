use std::{ops::Deref, str::Bytes, sync::Arc};

use anyhow::Result;
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
    let body = base64::engine::general_purpose::STANDARD.decode(body).unwrap();

    println!("body: {:?}", body.len());

    Response::builder()
        .body(Body::from_bytes(body.clone().into()))
        .set_content_type("image/jpeg")
        .into_response()
}
