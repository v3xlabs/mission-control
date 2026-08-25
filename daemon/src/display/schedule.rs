use chrono::{Datelike, Local, NaiveTime, Timelike};

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
    let now = at.time();

    for window in windows {
        if !window.days.contains(&today) {
            continue;
        }

        let (Some(from), Some(to)) = (parse_time(&window.from), parse_time(&window.to)) else {
            continue;
        };

        // A window that ends before it starts runs past midnight.
        let inside = if from <= to {
            now >= from && now < to
        } else {
            now >= from || now < to
        };

        if inside {
            return Baseline::On;
        }
    }

    Baseline::Off
}

pub fn seconds_until_next_boundary(windows: &[ScheduleWindow], at: chrono::DateTime<Local>) -> u64 {
    let now = at.time().num_seconds_from_midnight() as i64;

    let next = windows
        .iter()
        .flat_map(|window| [parse_time(&window.from), parse_time(&window.to)])
        .flatten()
        .map(|time| time.num_seconds_from_midnight() as i64)
        .filter(|seconds| *seconds > now)
        .min();

    match next {
        Some(seconds) => (seconds - now) as u64,
        None => (86_400 - now) as u64,
    }
}

fn parse_time(text: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(text, "%H:%M").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn window(days: &[Weekday], from: &str, to: &str) -> ScheduleWindow {
        ScheduleWindow {
            days: days.to_vec(),
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    /// 2026-08-24 is a Monday.
    fn monday_at(hour: u32, minute: u32) -> chrono::DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 8, 24, hour, minute, 0)
            .single()
            .expect("valid local time")
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
        assert_eq!(baseline_at(&windows, monday_at(1, 0)), Baseline::On);
        assert_eq!(baseline_at(&windows, monday_at(12, 0)), Baseline::Off);
    }

    #[test]
    fn a_malformed_time_does_not_turn_the_screen_on() {
        let windows = [window(&[Weekday::Mon], "half past seven", "23:00")];

        assert_eq!(baseline_at(&windows, monday_at(9, 0)), Baseline::Off);
    }

    #[test]
    fn the_next_boundary_is_the_next_edge_today() {
        let windows = [window(&[Weekday::Mon], "07:30", "23:00")];

        assert_eq!(
            seconds_until_next_boundary(&windows, monday_at(7, 0)),
            30 * 60
        );
    }

    #[test]
    fn past_the_last_edge_the_boundary_is_tomorrow() {
        let windows = [window(&[Weekday::Mon], "07:30", "23:00")];
        let seconds = seconds_until_next_boundary(&windows, monday_at(23, 30));

        assert_eq!(seconds, 30 * 60);
    }
}
