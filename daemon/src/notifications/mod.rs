pub mod level;
pub mod notification;
pub mod sidebar;

pub use level::Level;
pub use notification::Notification;
pub use sidebar::Sidebar;

use std::time::{Duration, Instant};

use tokio::sync::{watch, Mutex};

/// What the alert pages read. They are dumb: the daemon decides what is showing and for how long,
/// and a page renders whatever it is handed.
pub struct Notifications {
    active: Mutex<Vec<Held>>,
    next: Mutex<u64>,
    changed: watch::Sender<u64>,
}

struct Held {
    notification: Notification,
    expires_at: Instant,
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(Vec::new()),
            next: Mutex::new(1),
            changed: watch::channel(0).0,
        }
    }

    /// Bumped whenever the list changes, so a page can wait rather than poll.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.changed.subscribe()
    }

    pub async fn push(&self, mut notification: Notification, lifetime: Duration) -> u64 {
        let notification_id = {
            let mut next = self.next.lock().await;
            let id = *next;

            *next += 1;

            id
        };

        notification.notification_id = notification_id;

        self.active.lock().await.push(Held {
            notification,
            expires_at: Instant::now() + lifetime,
        });

        self.changed.send_modify(|version| *version += 1);

        notification_id
    }

    pub async fn dismiss(&self, notification_id: u64) -> bool {
        let mut active = self.active.lock().await;
        let before = active.len();

        active.retain(|held| held.notification.notification_id != notification_id);

        let removed = active.len() != before;

        if removed {
            self.changed.send_modify(|version| *version += 1);
        }

        removed
    }

    /// Everything still live, newest last, with each one's remaining time recomputed.
    pub async fn active(&self) -> Vec<Notification> {
        let now = Instant::now();
        let mut active = self.active.lock().await;
        let before = active.len();

        active.retain(|held| held.expires_at > now);

        if active.len() != before {
            self.changed.send_modify(|version| *version += 1);
        }

        active
            .iter()
            .map(|held| Notification {
                expires_in_seconds: held.expires_at.saturating_duration_since(now).as_secs(),
                ..held.notification.clone()
            })
            .collect()
    }

    /// The one a takeover should be showing, which is the most recent that has not expired.
    pub async fn current(&self) -> Option<Notification> {
        self.active().await.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NotificationMode;

    fn any(title: &str) -> Notification {
        Notification {
            notification_id: 0,
            title: title.to_string(),
            body: None,
            level: Level::Info,
            mode: NotificationMode::Takeover,
            expires_in_seconds: 0,
            tab_id: None,
            stinger: None,
        }
    }

    #[tokio::test]
    async fn ids_are_handed_out_in_order() {
        let notifications = Notifications::new();

        assert_eq!(notifications.push(any("one"), Duration::from_secs(60)).await, 1);
        assert_eq!(notifications.push(any("two"), Duration::from_secs(60)).await, 2);
    }

    #[tokio::test]
    async fn the_current_one_is_the_newest() {
        let notifications = Notifications::new();

        notifications.push(any("one"), Duration::from_secs(60)).await;
        notifications.push(any("two"), Duration::from_secs(60)).await;

        assert_eq!(notifications.current().await.unwrap().title, "two");
    }

    #[tokio::test]
    async fn an_expired_notification_stops_being_current() {
        let notifications = Notifications::new();

        notifications.push(any("gone"), Duration::from_millis(1)).await;
        tokio::time::sleep(Duration::from_millis(20)).await;

        assert!(notifications.current().await.is_none());
    }

    #[tokio::test]
    async fn dismissing_reports_whether_it_was_there() {
        let notifications = Notifications::new();
        let id = notifications.push(any("one"), Duration::from_secs(60)).await;

        assert!(notifications.dismiss(id).await);
        assert!(!notifications.dismiss(id).await);
    }

    #[tokio::test]
    async fn remaining_time_counts_down() {
        let notifications = Notifications::new();

        notifications.push(any("one"), Duration::from_secs(30)).await;

        let remaining = notifications.current().await.unwrap().expires_in_seconds;

        assert!((28..=30).contains(&remaining));
    }
}
