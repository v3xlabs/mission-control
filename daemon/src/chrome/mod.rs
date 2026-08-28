pub mod capture;
pub mod controller;
pub mod message;
pub mod preview;
pub mod profile;
pub mod request;
pub mod response;
pub mod state;

pub use controller::ChromeController;
pub use message::ChromeMessage;
pub use preview::Preview;
pub use profile::mark_clean_exit;
pub use request::Request;
pub use response::ChromeResponse;
pub use state::ChromeState;

use std::time::Duration;

use anyhow::{anyhow, Result};

const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn ask(controller: &ChromeController, message: ChromeMessage) -> Result<ChromeResponse> {
    if !controller.is_running() {
        return Err(anyhow!("the browser is not running"));
    }

    let (reply, answer) = tokio::sync::oneshot::channel();

    controller
        .sender()
        .send(Request { message, reply })
        .await
        .map_err(|_| anyhow!("chrome controller is not running"))?;

    match tokio::time::timeout(REPLY_TIMEOUT, answer).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(_)) => Err(anyhow!("chrome controller dropped the request")),
        Err(_) => Err(anyhow!("chrome controller did not answer in time")),
    }
}

pub async fn tell(controller: &ChromeController, message: ChromeMessage) -> Result<()> {
    match ask(controller, message).await? {
        ChromeResponse::Error { message } => Err(anyhow!(message)),
        _ => Ok(()),
    }
}
