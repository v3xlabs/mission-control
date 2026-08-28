pub mod feed;
pub mod meeting;
pub mod occurrence;

pub use meeting::Meeting;
pub use occurrence::Occurrence;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use chrono::{DateTime, Local};
use icalendar::Calendar as IcalCalendar;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    config::{Calendar, MeetingsConfig, NotificationMode},
    notifications::{Level, Notification},
    state::AppState,
};

use self::feed::Freshness;

const KEY_PREFIX: &str = "calendar:";

#[derive(Default)]
pub struct Feeds {
    parsed: Mutex<HashMap<String, Parsed>>,
}

struct Parsed {
    calendar: IcalCalendar,
    fetched_at: Instant,
    stale: bool,
    last_attempt_failed: bool,
}

pub async fn run(app_state: Arc<AppState>) {
    let feeds = Feeds::default();

    loop {
        let config = app_state.config.read().await.calendars;

        if config.calendars.is_empty() {
            tokio::time::sleep(config.poll.into()).await;

            continue;
        }

        reconcile(&app_state, &feeds, &config.meetings, &config.calendars).await;
        tokio::time::sleep(config.poll.into()).await;
    }
}

async fn reconcile(
    app_state: &Arc<AppState>,
    feeds: &Feeds,
    meetings: &MeetingsConfig,
    calendars: &[Calendar],
) {
    let now = Local::now();
    let mut wanted = HashSet::new();

    for calendar in calendars {
        let Some(reading) = read(app_state, feeds, calendar).await else {
            continue;
        };

        let until = now + chrono::Duration::from_std(calendar.window.into()).unwrap_or_default();
        let occurrences = occurrence::expand(&reading.calendar, meetings, now, until);

        for occurrence in &occurrences {
            wanted.insert(push_entry(app_state, calendar, occurrence, now).await);

            for lead in &calendar.leads {
                if let Some(key) = push_toast(app_state, calendar, occurrence, *lead, now).await {
                    wanted.insert(key);
                }
            }
        }

        if reading.stale {
            wanted.insert(push_stale_warning(app_state, calendar, now).await);
        }
    }

    app_state
        .notifications
        .retain_keyed(|key| !key.starts_with(KEY_PREFIX) || wanted.contains(key))
        .await;
}

struct Reading {
    calendar: IcalCalendar,
    stale: bool,
}

async fn read(app_state: &Arc<AppState>, feeds: &Feeds, calendar: &Calendar) -> Option<Reading> {
    let mut parsed = feeds.parsed.lock().await;

    let due = parsed.get(&calendar.calendar_id).is_none_or(|held| {
        held.last_attempt_failed || held.fetched_at.elapsed() >= calendar.refresh.into()
    });

    if due {
        match feed::load(calendar, &app_state.config.dirs.cache).await {
            Freshness::Fresh(fetched) => {
                parsed.insert(
                    calendar.calendar_id.clone(),
                    Parsed {
                        calendar: fetched,
                        fetched_at: Instant::now(),
                        stale: false,
                        last_attempt_failed: false,
                    },
                );
            }
            Freshness::Cached(fetched, age) => {
                let stale = age > calendar.refresh.into();

                warn!(
                    calendar.calendar_id,
                    age_seconds = age.as_secs(),
                    stale,
                    "showing a cached calendar"
                );

                parsed.insert(
                    calendar.calendar_id.clone(),
                    Parsed {
                        calendar: fetched,
                        fetched_at: Instant::now(),
                        stale,
                        last_attempt_failed: true,
                    },
                );
            }
            Freshness::Missing(reason) => {
                warn!(calendar.calendar_id, "no calendar to show: {reason}");
                parsed.remove(&calendar.calendar_id);

                return None;
            }
        }
    }

    let held = parsed.get(&calendar.calendar_id)?;

    Some(Reading {
        calendar: held.calendar.clone(),
        stale: held.stale,
    })
}

async fn push_entry(
    app_state: &Arc<AppState>,
    calendar: &Calendar,
    occurrence: &Occurrence,
    now: DateTime<Local>,
) -> String {
    let key = occurrence.key(&calendar.calendar_id);

    app_state
        .notifications
        .push(
            Notification {
                notification_id: 0,
                key: Some(key.clone()),
                title: occurrence.summary.clone(),
                body: None,
                level: Level::Info,
                mode: NotificationMode::Sidebar,
                expires_in_seconds: 0,
                starts_at: Some(occurrence.start),
                ends_at: Some(occurrence.end),
                location: occurrence.location.clone(),
                meeting: occurrence.meeting.clone(),
                tab_id: None,
                stinger: None,
            },
            remaining(now, occurrence.end),
        )
        .await;

    key
}

/// The window a toast is up for is absolute, `start - lead` to `toast_duration` later, rather than
/// a crossing this loop has to remember. That is what makes firing once free and what lets a
/// restart mid-window arrive at the same answer as a daemon that has been running all morning.
async fn push_toast(
    app_state: &Arc<AppState>,
    calendar: &Calendar,
    occurrence: &Occurrence,
    lead: crate::config::HumanDuration,
    now: DateTime<Local>,
) -> Option<String> {
    let lead_duration = chrono::Duration::from_std(lead.into()).ok()?;
    let shows_at = occurrence.start - lead_duration;
    let hides_at = shows_at + chrono::Duration::from_std(calendar.toast_duration.into()).ok()?;

    if now < shows_at || now >= hides_at {
        return None;
    }

    let key = format!(
        "{}:lead:{}",
        occurrence.key(&calendar.calendar_id),
        lead.seconds()
    );

    app_state
        .notifications
        .push(
            Notification {
                notification_id: 0,
                key: Some(key.clone()),
                title: occurrence.summary.clone(),
                body: occurrence.location.clone(),
                level: Level::Warning,
                mode: NotificationMode::Toast,
                expires_in_seconds: 0,
                starts_at: Some(occurrence.start),
                ends_at: Some(occurrence.end),
                location: None,
                meeting: occurrence.meeting.clone(),
                tab_id: None,
                stinger: None,
            },
            remaining(now, hides_at),
        )
        .await;

    Some(key)
}

async fn push_stale_warning(
    app_state: &Arc<AppState>,
    calendar: &Calendar,
    now: DateTime<Local>,
) -> String {
    let key = format!("{KEY_PREFIX}{}:stale", calendar.calendar_id);

    app_state
        .notifications
        .push(
            Notification {
                notification_id: 0,
                key: Some(key.clone()),
                title: format!("{} is out of date", calendar.display_name()),
                body: Some("the calendar server is not answering".to_string()),
                level: Level::Warning,
                mode: NotificationMode::Sidebar,
                expires_in_seconds: 0,
                starts_at: None,
                ends_at: None,
                location: None,
                meeting: None,
                tab_id: None,
                stinger: None,
            },
            remaining(now, now + chrono::Duration::minutes(30)),
        )
        .await;

    key
}

fn remaining(now: DateTime<Local>, until: DateTime<Local>) -> std::time::Duration {
    (until - now).to_std().unwrap_or_default()
}

pub async fn agenda(app_state: &Arc<AppState>) -> Vec<Notification> {
    app_state
        .notifications
        .active()
        .await
        .into_iter()
        .filter(|notification| {
            notification
                .key
                .as_deref()
                .is_some_and(|key| key.starts_with(KEY_PREFIX))
                && notification.mode == NotificationMode::Sidebar
        })
        .collect()
}

impl Feeds {
    pub async fn invalidate(&self) {
        self.parsed.lock().await.clear();
        info!("calendar feeds will be fetched again on the next poll");
    }
}
