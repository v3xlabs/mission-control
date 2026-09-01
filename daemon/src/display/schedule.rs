use std::cmp::Ordering;

use chrono::{Datelike, Local};

use crate::config::{ScheduleWindow, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Baseline {
    Unscheduled,
    On,
    Off,
}

pub fn baseline_at(windows: &[ScheduleWindow], at: chrono::DateTime<Local>) -> Baseline {
    if windows.is_empty() {
        return Baseline::Unscheduled;
    }

    let today = Weekday::from_chrono(at.weekday());
    let yesterday = Weekday::from_chrono(at.weekday().pred());
    let now = at.time();

    for window in windows {
        let inside = match window.from.cmp(&window.to) {
            Ordering::Less => window.days.contains(&today) && now >= window.from && now < window.to,
            // A window that ends before it starts runs past midnight, and the hours after midnight
            // belong to the evening that opened them rather than to the day the clock now names.
            // Weekdays 09:00 to 04:00 has to end on Saturday morning, not stop at Friday midnight.
            Ordering::Greater => {
                (window.days.contains(&today) && now >= window.from)
                    || (window.days.contains(&yesterday) && now < window.to)
            }
            // A window that ends where it starts is the whole day, which is otherwise something
            // `HH:MM` cannot say.
            Ordering::Equal => window.days.contains(&today),
        };

        if inside {
            return Baseline::On;
        }
    }

    Baseline::Off
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, TimeZone};

    fn at_time(text: &str) -> NaiveTime {
        NaiveTime::parse_from_str(text, "%H:%M").expect("a time of day")
    }

    fn window(days: &[Weekday], from: &str, to: &str) -> ScheduleWindow {
        ScheduleWindow {
            days: days.to_vec(),
            from: at_time(from),
            to: at_time(to),
        }
    }

    /// 2026-08-24 is a Monday, so the week runs from day 24 to day 30.
    fn day_at(day: u32, hour: u32, minute: u32) -> chrono::DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 8, day, hour, minute, 0)
            .single()
            .expect("valid local time")
    }

    fn monday_at(hour: u32, minute: u32) -> chrono::DateTime<Local> {
        day_at(24, hour, minute)
    }

    fn saturday_at(hour: u32, minute: u32) -> chrono::DateTime<Local> {
        day_at(29, hour, minute)
    }

    #[test]
    fn no_windows_means_no_opinion() {
        assert_eq!(baseline_at(&[], monday_at(3, 0)), Baseline::Unscheduled);
    }

    #[test]
    fn inside_a_weekday_window_is_on() {
        let windows = [window(&[Weekday::Mon, Weekday::Fri], "07:30", "23:00")];

        assert_eq!(baseline_at(&windows, monday_at(9, 0)), Baseline::On);
    }

    #[test]
    fn outside_a_weekday_window_is_off() {
        let windows = [window(&[Weekday::Mon], "07:30", "23:00")];

        assert_eq!(baseline_at(&windows, monday_at(2, 0)), Baseline::Off);
        assert_eq!(baseline_at(&windows, monday_at(23, 30)), Baseline::Off);
    }

    #[test]
    fn a_day_with_no_window_is_off() {
        let windows = [window(&[Weekday::Sat, Weekday::Sun], "09:00", "18:00")];

        assert_eq!(baseline_at(&windows, monday_at(12, 0)), Baseline::Off);
    }

    #[test]
    fn a_window_may_run_past_midnight() {
        let windows = [window(&[Weekday::Mon], "22:00", "02:00")];

        assert_eq!(baseline_at(&windows, monday_at(23, 0)), Baseline::On);
        assert_eq!(baseline_at(&windows, day_at(25, 1, 0)), Baseline::On);
        assert_eq!(baseline_at(&windows, monday_at(12, 0)), Baseline::Off);
    }

    /// The reason the day set names the day a window starts on: a Monday morning is not on unless
    /// Sunday evening was.
    #[test]
    fn the_hours_after_midnight_belong_to_the_evening_that_opened_them() {
        let weekdays = [window(
            &[
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            "09:00",
            "04:00",
        )];

        assert_eq!(baseline_at(&weekdays, saturday_at(2, 0)), Baseline::On);
        assert_eq!(baseline_at(&weekdays, saturday_at(4, 0)), Baseline::Off);
        assert_eq!(baseline_at(&weekdays, saturday_at(10, 0)), Baseline::Off);
        assert_eq!(baseline_at(&weekdays, monday_at(2, 0)), Baseline::Off);
        assert_eq!(baseline_at(&weekdays, monday_at(9, 0)), Baseline::On);
    }

    #[test]
    fn a_window_that_ends_where_it_starts_is_the_whole_day() {
        let windows = [window(&[Weekday::Sat], "00:00", "00:00")];

        assert_eq!(baseline_at(&windows, saturday_at(0, 0)), Baseline::On);
        assert_eq!(baseline_at(&windows, saturday_at(23, 59)), Baseline::On);
        assert_eq!(baseline_at(&windows, monday_at(12, 0)), Baseline::Off);
    }
}
