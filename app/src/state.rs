use std::sync::Arc;

use crate::chrome::ChromeController;

pub struct AppState {
    pub chrome: Arc<ChromeController>,
}
