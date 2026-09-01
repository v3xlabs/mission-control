use std::collections::HashSet;

use chrono::{DateTime, Duration, Local, TimeZone as _};
use icalendar::{Calendar, Component as _, DatePerhapsTime, Event, EventLike as _};
use tracing::{debug, warn};

use crate::config::MeetingsConfig;

use super::{meeting, Meeting, KEY_PREFIX};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub uid: String,
    pub summary: String,
    pub location: Option<String>,
    pub meeting: Option<Meeting>,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl Occurrence {
    /// `UID` is shared across a recurrence set, so the start is what separates one week of a
    /// series from the next.
    pub fn key(&self, calendar_id: &str) -> String {
        format!(
            "{KEY_PREFIX}{calendar_id}:{}:{}",
            self.uid,
            self.start.to_rfc3339()
        )
    }

    /// Two feeds that both carry one meeting hold it under two calendar ids, and an invitation
    /// exported twice does not keep its `UID` either, so what a viewer can see is what decides
    /// that two entries are one meeting.
    pub fn fingerprint(&self) -> Fingerprint<'_> {
        (
            self.summary.as_str(),
            self.start,
            self.end,
            self.location.as_deref(),
            self.meeting.as_ref().map(|meeting| meeting.url.as_str()),
        )
    }
}

type Fingerprint<'a> = (
    &'a str,
    DateTime<Local>,
    DateTime<Local>,
    Option<&'a str>,
    Option<&'a str>,
);

/// A rule with no `UNTIL` and no `COUNT` is infinite, so the expansion needs a bound of its own.
const LIMIT: u16 = 512;

/// Expansion runs in the feed's own timezone. Doing it in UTC and converting afterwards drifts by
/// an hour at each daylight saving boundary, so a standup that is 09:00 all year starts reading as
/// 08:00 from late October.
pub fn expand(
    calendar: &Calendar,
    meetings: &MeetingsConfig,
    from: DateTime<Local>,
    until: DateTime<Local>,
) -> Vec<Occurrence> {
    let mut occurrences = Vec::new();
    let mut unreadable = 0usize;
    let replaced = rescheduled_instances(calendar);

    for event in calendar.calendar_events() {
        let Some(uid) = event.get_uid() else {
            continue;
        };

        let summary = event.get_summary().unwrap_or("Untitled").to_string();
        let length = length_of(event.event());
        let meeting = meeting::find(event.event(), meetings);
        let location = event.get_location().map(str::to_string).filter(|location| {
            meeting
                .as_ref()
                .is_none_or(|meeting| meeting.url.trim() != location.trim())
        });
        let is_override = event.properties().contains_key(RECURRENCE_ID);

        // Google leaves `UNTIL` before `DTSTART` on old series, which no rule engine accepts and
        // which describes no occurrence anyway.
        let set = match event.get_recurrence() {
            Ok(set) => set,
            Err(error) => {
                debug!(uid, "cannot read the recurrence of an event: {error}");
                unreadable += 1;

                continue;
            }
        };

        // `after` and `before` are exclusive, so the near end is widened by the event's own length
        // to keep a meeting that is already running.
        let result = set
            .after(with_zone(from - length))
            .before(with_zone(until))
            .all(LIMIT);

        if result.limited {
            warn!(
                uid,
                "recurrence expansion hit its limit, some occurrences are not shown"
            );
        }

        for start in result.dates {
            let start = start.with_timezone(&Local);

            if !is_override && replaced.contains(&(uid.to_string(), start)) {
                continue;
            }

            occurrences.push(Occurrence {
                uid: uid.to_string(),
                summary: summary.clone(),
                location: location.clone(),
                meeting: meeting.clone(),
                start,
                end: start + length,
            });
        }
    }

    if unreadable > 0 {
        warn!(
            unreadable,
            "events whose recurrence rule could not be read were left off the rail"
        );
    }

    occurrences.sort_by_key(|occurrence| occurrence.start);
    occurrences
}

const RECURRENCE_ID: &str = "RECURRENCE-ID";

/// RFC 5545 §3.8.4.4: a `VEVENT` carrying `RECURRENCE-ID` is one changed meeting of a series,
/// under the same `UID`, naming the start the series would otherwise have generated. Expanding
/// every component independently shows that meeting twice, once where it was and once where it
/// moved to.
fn rescheduled_instances(calendar: &Calendar) -> HashSet<(String, DateTime<Local>)> {
    let mut replaced = HashSet::new();

    for event in calendar.calendar_events() {
        let Some(uid) = event.get_uid() else {
            continue;
        };

        let Some(property) = event.properties().get(RECURRENCE_ID) else {
            continue;
        };

        if let Some(when) = DatePerhapsTime::from_property(property).and_then(to_local) {
            replaced.insert((uid.to_string(), when));
        }
    }

    replaced
}

/// `DTEND` belongs to the first instance, so it is read as a length and applied to every
/// occurrence rather than used as an end date.
fn length_of(event: &Event) -> Duration {
    let start = event.get_start().and_then(to_local);
    let end = event.get_end().and_then(to_local);

    match (start, end) {
        (Some(start), Some(end)) if end > start => end - start,
        _ => Duration::hours(1),
    }
}

fn to_local(value: DatePerhapsTime) -> Option<DateTime<Local>> {
    match value {
        DatePerhapsTime::DateTime(date_time) => date_time.try_into_utc().map(|utc| utc.into()),
        DatePerhapsTime::Date(date) => Local
            .from_local_datetime(&date.and_hms_opt(0, 0, 0)?)
            .single(),
    }
}

fn with_zone(value: DateTime<Local>) -> DateTime<icalendar::Tz> {
    value.with_timezone(&icalendar::Tz::LOCAL)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(body: &str) -> Calendar {
        body.parse::<Calendar>().unwrap()
    }

    fn at(text: &str) -> DateTime<Local> {
        DateTime::parse_from_rfc3339(text).unwrap().into()
    }

    fn wrap(body: &str) -> String {
        format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n{body}END:VCALENDAR\r\n")
    }

    #[test]
    fn a_single_event_produces_one_occurrence() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:one\r\n\
             SUMMARY:Design review\r\n\
             LOCATION:Room 4\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T140000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T150000\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-02T00:00:00+01:00"),
            at("2026-03-03T00:00:00+01:00"),
        );

        assert_eq!(occurrences.len(), 1);
        assert_eq!(occurrences[0].summary, "Design review");
        assert_eq!(occurrences[0].location.as_deref(), Some("Room 4"));
        assert_eq!(
            occurrences[0].end - occurrences[0].start,
            Duration::hours(1)
        );
    }

    #[test]
    fn a_weekly_rule_keeps_its_local_time_across_a_daylight_saving_boundary() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:standup\r\n\
             SUMMARY:Standup\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T090000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T091500\r\n\
             RRULE:FREQ=WEEKLY;BYDAY=MO\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-01T00:00:00+01:00"),
            at("2026-04-14T00:00:00+02:00"),
        );

        let local_times: Vec<_> = occurrences
            .iter()
            .map(|occurrence| occurrence.start.format("%H:%M").to_string())
            .collect();

        assert!(
            occurrences.len() >= 5,
            "expected several Mondays, got {occurrences:?}"
        );
        assert!(
            local_times.iter().all(|time| time == "09:00"),
            "the meeting drifted: {local_times:?}"
        );
    }

    #[test]
    fn a_location_that_is_the_join_link_is_not_repeated() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:one\r\n\
             SUMMARY:Standup\r\n\
             LOCATION:https://us02web.zoom.us/j/81891522989\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T090000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T091500\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-02T00:00:00+01:00"),
            at("2026-03-03T00:00:00+01:00"),
        );

        assert_eq!(occurrences[0].location, None);
        assert_eq!(
            occurrences[0].meeting.as_ref().unwrap().provider.as_deref(),
            Some("zoom")
        );
    }

    #[test]
    fn a_location_that_is_a_room_is_kept_beside_the_meeting() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:one\r\n\
             SUMMARY:Design review\r\n\
             LOCATION:Room 4\r\n\
             X-GOOGLE-CONFERENCE:https://meet.google.com/jwc-vnyk-izt\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T090000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T091500\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-02T00:00:00+01:00"),
            at("2026-03-03T00:00:00+01:00"),
        );

        assert_eq!(occurrences[0].location.as_deref(), Some("Room 4"));
        assert_eq!(
            occurrences[0].meeting.as_ref().unwrap().provider.as_deref(),
            Some("meet")
        );
    }

    #[test]
    fn an_excluded_date_is_not_an_occurrence() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:standup\r\n\
             SUMMARY:Standup\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T090000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T091500\r\n\
             RRULE:FREQ=DAILY;COUNT=3\r\n\
             EXDATE;TZID=Europe/Amsterdam:20260303T090000\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-01T00:00:00+01:00"),
            at("2026-03-06T00:00:00+01:00"),
        );

        let days: Vec<_> = occurrences
            .iter()
            .map(|occurrence| occurrence.start.format("%d").to_string())
            .collect();

        assert_eq!(days, ["02", "04"]);
    }

    #[test]
    fn an_occurrence_already_running_is_still_returned() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:one\r\n\
             SUMMARY:Long meeting\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T090000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T110000\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-02T10:00:00+01:00"),
            at("2026-03-02T23:00:00+01:00"),
        );

        assert_eq!(occurrences.len(), 1);
    }

    #[test]
    fn a_rescheduled_instance_replaces_the_one_the_series_would_have_generated() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:series@example.com\r\n\
             SUMMARY:SSZ Engine API\r\n\
             DTSTART:20260724T130000Z\r\n\
             DTEND:20260724T140000Z\r\n\
             RRULE:FREQ=WEEKLY;UNTIL=20270120T130000Z\r\n\
             END:VEVENT\r\n\
             BEGIN:VEVENT\r\n\
             UID:series@example.com\r\n\
             SUMMARY:SSZ Engine API\r\n\
             RECURRENCE-ID:20260828T130000Z\r\n\
             DTSTART:20260828T090000Z\r\n\
             DTEND:20260828T100000Z\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-08-28T00:00:00Z"),
            at("2026-08-29T00:00:00Z"),
        );

        let starts: Vec<_> = occurrences
            .iter()
            .map(|occurrence| occurrence.start.to_utc().format("%H:%M").to_string())
            .collect();

        assert_eq!(
            starts,
            ["09:00"],
            "the meeting moved, so 13:00 is not a meeting"
        );
    }

    #[test]
    fn the_key_separates_two_occurrences_of_one_event() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:standup\r\n\
             SUMMARY:Standup\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T090000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T091500\r\n\
             RRULE:FREQ=DAILY;COUNT=2\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-01T00:00:00+01:00"),
            at("2026-03-06T00:00:00+01:00"),
        );

        assert_ne!(occurrences[0].key("work"), occurrences[1].key("work"));
    }

    #[test]
    fn one_meeting_read_from_two_feeds_has_one_fingerprint() {
        let event = |uid: &str| {
            format!(
                "BEGIN:VEVENT\r\n\
                 UID:{uid}\r\n\
                 SUMMARY:Design review\r\n\
                 LOCATION:Room 4\r\n\
                 DTSTART;TZID=Europe/Amsterdam:20260302T140000\r\n\
                 DTEND;TZID=Europe/Amsterdam:20260302T150000\r\n\
                 END:VEVENT\r\n"
            )
        };

        let read = |uid: &str| {
            expand(
                &parse(&wrap(&event(uid))),
                &MeetingsConfig::default(),
                at("2026-03-02T00:00:00+01:00"),
                at("2026-03-03T00:00:00+01:00"),
            )
        };

        assert_eq!(
            read("work")[0].fingerprint(),
            read("personal")[0].fingerprint()
        );
    }

    #[test]
    fn two_meetings_at_one_time_have_two_fingerprints() {
        let calendar = parse(&wrap(
            "BEGIN:VEVENT\r\n\
             UID:one\r\n\
             SUMMARY:Design review\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T140000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T150000\r\n\
             END:VEVENT\r\n\
             BEGIN:VEVENT\r\n\
             UID:two\r\n\
             SUMMARY:Standup\r\n\
             DTSTART;TZID=Europe/Amsterdam:20260302T140000\r\n\
             DTEND;TZID=Europe/Amsterdam:20260302T150000\r\n\
             END:VEVENT\r\n",
        ));

        let occurrences = expand(
            &calendar,
            &MeetingsConfig::default(),
            at("2026-03-02T00:00:00+01:00"),
            at("2026-03-03T00:00:00+01:00"),
        );

        assert_ne!(occurrences[0].fingerprint(), occurrences[1].fingerprint());
    }
}
