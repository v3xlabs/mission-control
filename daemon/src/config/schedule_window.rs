use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use super::Weekday;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduleWindow {
    /// The days the window *starts* on. A window that runs past midnight belongs to the day it
    /// began, so a Friday evening still ends on Saturday morning.
    pub days: Vec<Weekday>,
    /// `HH:MM`, local time.
    #[serde(with = "hour_minute")]
    pub from: NaiveTime,
    #[serde(with = "hour_minute")]
    pub to: NaiveTime,
}

/// A time is read at load, so a typo stops the daemon with the file it is in rather than leaving
/// the screen dark all day. It is written back as `07:30`, not as chrono's `07:30:00`.
mod hour_minute {
    use chrono::NaiveTime;
    use serde::{de::Error as _, Deserialize as _, Deserializer, Serializer};

    const FORMAT: &str = "%H:%M";

    pub fn serialize<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&time.format(FORMAT).to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveTime, D::Error> {
        let text = String::deserialize(deserializer)?;

        NaiveTime::parse_from_str(&text, FORMAT)
            .map_err(|_| D::Error::custom(format!("{text:?} is not a time of day, expected HH:MM")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    struct Document {
        schedule: Vec<ScheduleWindow>,
    }

    #[test]
    fn a_window_survives_a_round_trip_as_hours_and_minutes() {
        let document: Document =
            toml::from_str("[[schedule]]\ndays = [\"mon\"]\nfrom = \"07:30\"\nto = \"23:00\"\n")
                .expect("a valid window");

        assert_eq!(
            document.schedule[0].from,
            NaiveTime::from_hms_opt(7, 30, 0).unwrap()
        );

        let written = toml::to_string(&document).expect("a window that can be written");

        assert!(written.contains("from = \"07:30\""), "{written}");
    }

    #[test]
    fn a_time_that_is_not_a_time_is_a_load_error() {
        let result = toml::from_str::<Document>(
            "[[schedule]]\ndays = [\"mon\"]\nfrom = \"half past seven\"\nto = \"23:00\"\n",
        );

        assert!(result.is_err(), "a typo has to be reported, not ignored");
    }
}
