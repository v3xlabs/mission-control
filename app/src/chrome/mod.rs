pub mod messages;
pub mod controller;

pub use controller::ChromeController;
pub use messages::*;

use anyhow::Result;
use std::sync::Arc;
use futures::SinkExt;

use crate::{
    config::ChromiumConfig,
    state::AppState,
};

/// Legacy interface for backward compatibility
pub async fn start_chrome_controller(config: &ChromiumConfig, state: &Arc<AppState>) -> Result<Arc<ChromeController>> {
    let controller = Arc::new(ChromeController::new());
    controller.start(config, state).await?;
    Ok(controller)
}

/// Send a message to the Chrome controller
pub async fn send_chrome_message(controller: &ChromeController, message: ChromeMessage) -> Result<()> {
    tracing::info!("Sending message to Chrome controller: {:?}", message);
    let mut sender = controller.get_message_sender();
    sender.send(message).await.map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
    tracing::info!("Message sent successfully to Chrome controller");
    Ok(())
}

/// Activate a playlist by ID
pub async fn activate_playlist(controller: &ChromeController, playlist_id: String) -> Result<()> {
    send_chrome_message(controller, ChromeMessage::ActivatePlaylist { playlist_id }).await
}

/// Activate a specific tab
pub async fn activate_tab(controller: &ChromeController, tab_id: String, playlist_id: String) -> Result<()> {
    send_chrome_message(controller, ChromeMessage::ActivateTab { tab_id, playlist_id }).await
}

/// Get current Chrome status
pub async fn get_chrome_status(controller: &ChromeController) -> Result<()> {
    send_chrome_message(controller, ChromeMessage::GetStatus).await
}

/// Stop current playlist
pub async fn stop_playlist(controller: &ChromeController) -> Result<()> {
    send_chrome_message(controller, ChromeMessage::StopPlaylist).await
}

/// Start playlist rotation
pub async fn start_playlist(controller: &ChromeController) -> Result<()> {
    send_chrome_message(controller, ChromeMessage::StartPlaylist).await
} 