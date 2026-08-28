use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{Context as _, Result};
use icalendar::Calendar as IcalCalendar;
use tracing::{info, warn};

use crate::config::Calendar;

pub enum Freshness {
    Fresh(IcalCalendar),
    /// The duration is the age of the cache file it came from.
    Cached(IcalCalendar, Duration),
    Missing(String),
}

pub async fn load(calendar: &Calendar, cache: &Path) -> Freshness {
    let file = cache_file(cache, &calendar.calendar_id);

    match fetch(calendar).await {
        Ok(body) => {
            if let Err(error) = write_cache(&file, &body) {
                warn!(
                    calendar.calendar_id,
                    "cannot keep a copy of the feed: {error}"
                );
            }

            match body.parse::<IcalCalendar>() {
                Ok(parsed) => Freshness::Fresh(parsed),
                Err(reason) => fall_back(&file, &reason),
            }
        }
        Err(error) => {
            warn!(calendar.calendar_id, "cannot reach the feed: {error}");

            fall_back(&file, &error.to_string())
        }
    }
}

fn fall_back(file: &Path, reason: &str) -> Freshness {
    let Ok(body) = std::fs::read_to_string(file) else {
        return Freshness::Missing(reason.to_string());
    };

    match body.parse::<IcalCalendar>() {
        Ok(parsed) => Freshness::Cached(parsed, age_of(file)),
        Err(reason) => Freshness::Missing(reason),
    }
}

async fn fetch(calendar: &Calendar) -> Result<String> {
    let url = calendar.url.resolve()?;
    let response = reqwest::get(&url)
        .await
        .context("the request failed")?
        .error_for_status()
        .context("the feed answered with an error")?;

    let body = response.text().await.context("the feed sent no body")?;

    info!(
        calendar.calendar_id,
        bytes = body.len(),
        "fetched the calendar feed"
    );

    Ok(body)
}

fn cache_file(cache: &Path, calendar_id: &str) -> PathBuf {
    cache.join("calendars").join(format!("{calendar_id}.ics"))
}

fn write_cache(file: &Path, body: &str) -> Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let temporary = file.with_extension("ics.tmp");

    std::fs::write(&temporary, body)?;
    std::fs::rename(&temporary, file)?;

    Ok(())
}

fn age_of(file: &Path) -> Duration {
    std::fs::metadata(file)
        .and_then(|metadata| metadata.modified())
        .and_then(|modified| {
            SystemTime::now().duration_since(modified).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "the cache is from the future",
                )
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("missiond-test-feed-{name}"));

        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    const BODY: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nEND:VCALENDAR\r\n";

    #[test]
    fn a_cached_body_survives_a_write() {
        let file = scratch("write").join("personal.ics");

        write_cache(&file, BODY).unwrap();

        assert_eq!(std::fs::read_to_string(&file).unwrap(), BODY);
    }

    #[test]
    fn a_feed_that_has_never_worked_reports_that_rather_than_an_empty_calendar() {
        let file = scratch("missing").join("personal.ics");

        assert!(matches!(
            fall_back(&file, "the request failed"),
            Freshness::Missing(reason) if reason == "the request failed"
        ));
    }

    #[test]
    fn a_failed_fetch_falls_back_to_the_last_good_body() {
        let file = scratch("fallback").join("personal.ics");

        write_cache(&file, BODY).unwrap();

        assert!(matches!(
            fall_back(&file, "offline"),
            Freshness::Cached(_, _)
        ));
    }
}
