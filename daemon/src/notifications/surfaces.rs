use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::info;

use crate::{config::NotificationMode, state::AppState};

use super::{Sidebar, Toast};

pub struct Surfaces {
    pub sidebar: Sidebar,
    pub toast: Toast,
    manual: Mutex<Option<Manual>>,
}

struct Manual {
    open: bool,
    /// The newest notification when the button was pressed. Anything newer supersedes the choice,
    /// while an expiry does not, which is why this is an id rather than a change count.
    after_id: u64,
}

impl Default for Surfaces {
    fn default() -> Self {
        Self::new()
    }
}

impl Surfaces {
    pub fn new() -> Self {
        Self {
            sidebar: Sidebar::new(),
            toast: Toast::new(),
            manual: Mutex::new(None),
        }
    }

    pub async fn toggle_sidebar(&self, app_state: &Arc<AppState>) -> bool {
        let open = !self.sidebar.is_open().await;

        *self.manual.lock().await = Some(Manual {
            open,
            after_id: app_state.notifications.last_id().await,
        });

        info!(open, "sidebar toggled by hand");
        self.reconcile(app_state).await;

        open
    }

    pub async fn reconcile(&self, app_state: &Arc<AppState>) {
        let active = app_state.notifications.active().await;

        let newest_sidebar = active
            .iter()
            .filter(|notification| notification.mode == NotificationMode::Sidebar)
            .map(|notification| notification.notification_id)
            .max();

        let wanted = match self.manual.lock().await.as_ref() {
            Some(manual) if newest_sidebar.is_none_or(|id| id <= manual.after_id) => manual.open,
            _ => newest_sidebar.is_some(),
        };

        match (wanted, self.sidebar.is_open().await) {
            (true, false) => self.sidebar.open(app_state).await,
            (false, true) => self.sidebar.close().await,
            _ => {}
        }

        let toast_wanted = active
            .iter()
            .any(|notification| notification.mode == NotificationMode::Toast);

        match (toast_wanted, self.toast.is_open().await) {
            (true, false) => self.toast.open(app_state).await,
            (false, true) => self.toast.close(app_state).await,
            _ => {}
        }
    }

    pub async fn shutdown(&self, app_state: &Arc<AppState>) {
        self.sidebar.close().await;
        self.toast.close(app_state).await;
    }
}

/// Expiry reaches this loop only because the notification stream calls `Notifications::active`
/// on a timer. Nothing here notices a meeting ending on its own.
pub async fn run(app_state: Arc<AppState>) {
    let mut changed = app_state.notifications.subscribe();

    loop {
        app_state.surfaces.reconcile(&app_state).await;

        if changed.changed().await.is_err() {
            break;
        }
    }
}
