use tokio::sync::oneshot;

use super::{ChromeMessage, ChromeResponse};

#[derive(Debug)]
pub struct Request {
    pub message: ChromeMessage,
    pub reply: oneshot::Sender<ChromeResponse>,
}
