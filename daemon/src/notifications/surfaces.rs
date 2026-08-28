use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::info;

use crate::{config::NotificationMode, state::AppState};

use super::{Sidebar, Toast};

/// The windows that show notifications, kept matching the list.
///
/// One task decides whether each window is up, rather than every caller that raises an alert. A
/// per alert timer has to guess whether a later alert has already moved its deadline; asking the
/// list cannot be wrong, and a window that died is reopened by the next pass rather than never.
pub struct Surfaces {
    pub sidebar: Sidebar,
    pub toast: Toast,
    manual: Mutex<Option<Manual>>,
}

/// A rail opened or closed by hand, from the web UI or a button somewhere else.
struct Manual {
    open: bool,
    /// The newest notification that existed when the button was pressed.
    ///
    /// Anything newer than this supersedes the choice, so a rail closed by hand comes back for
    /// the next meeting rather than staying shut for the rest of the day. Expiry does not, which
    /// is why this counts notifications rather than changes.
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

    /// Opens the rail if it is closed, closes it if it is open, and reports where it ended up.
    ///
    /// This is the one call a button needs. Nothing else has to know what is on the rail, which is
    /// what makes it reachable from a launchpad key or a Home Assistant automation.
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

/// Follows the notification list for as long as the daemon runs.
///
/// Expiry is noticed by `Notifications::active`, which only prunes when it is called, so the
/// stream in the http layer is what makes an entry that ended on its own reach this loop.
pub async fn run(app_state: Arc<AppState>) {
    let mut changed = app_state.notifications.subscribe();

    loop {
        app_state.surfaces.reconcile(&app_state).await;

        if changed.changed().await.is_err() {
            break;
        }
    }
}
