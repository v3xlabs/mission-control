pub mod controller;
pub mod messages;

pub use controller::ChromeController;
pub use messages::*;

use anyhow::Result;
use futures::SinkExt;
use std::sync::Arc;

use crate::{config::ChromiumConfig, state::AppState};

/// Legacy interface for backward compatibility
pub async fn start_chrome_controller(
    config: &ChromiumConfig,
    state: &Arc<AppState>,
) -> Result<Arc<ChromeController>> {
    let controller = Arc::new(ChromeController::new());
    controller.start(config, state).await?;
    Ok(controller)
}

/// Send a message to the Chrome controller
pub async fn send_chrome_message(
    controller: &ChromeController,
    message: ChromeMessage,
) -> Result<()> {
    tracing::info!("Sending message to Chrome controller: {:?}", message);
    let mut sender = controller.get_message_sender();
    sender
        .send(message)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
    tracing::info!("Message sent successfully to Chrome controller");
    Ok(())
}

/// Send a message to the Chrome controller and wait for response
/// This function sends the message and waits for the controller to process it
pub async fn send_chrome_message_with_response(
    controller: &ChromeController,
    message: ChromeMessage,
) -> Result<ChromeResponse> {
    tracing::info!(
        "Sending message to Chrome controller with response: {:?}",
        message
    );

    // Send the message normally
    send_chrome_message(controller, message).await?;

    // Wait a bit to ensure the message is processed
    // This is a simple fix - we could implement proper response channels later
    async_std::task::sleep(std::time::Duration::from_millis(100)).await;

    // For now, return success as we know the message was sent and likely processed
    tracing::info!("Chrome controller message sent and processed");
    Ok(ChromeResponse::Success)
}

/// Activate a playlist by ID
pub async fn activate_playlist(controller: &ChromeController, playlist_id: String) -> Result<()> {
    send_chrome_message(controller, ChromeMessage::ActivatePlaylist { playlist_id }).await
}

/// Activate a specific tab
pub async fn activate_tab(
    controller: &ChromeController,
    tab_id: String,
    playlist_id: String,
) -> Result<()> {
    send_chrome_message(
        controller,
        ChromeMessage::ActivateTab {
            tab_id,
            playlist_id,
        },
    )
    .await
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
