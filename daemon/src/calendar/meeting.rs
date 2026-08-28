use icalendar::{Component as _, Event, EventLike as _};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::config::MeetingsConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Object)]
pub struct Meeting {
    pub url: String,
    /// A name from `meetings.providers` in `calendars.toml`, so the set is open, not fixed.
    #[serde(default)]
    pub provider: Option<String>,
}

/// `X-GOOGLE-CONFERENCE` is what Google Calendar writes for a Meet link; `CONFERENCE` is the
/// property RFC 7986 §5.11 defines for the same job.
const LINK_PROPERTIES: [&str; 2] = ["X-GOOGLE-CONFERENCE", "CONFERENCE"];

/// A Google Calendar `DESCRIPTION` leads with an issue link often enough that the first url in it
/// is the wrong one, so a url found there counts only when a provider claims it.
pub fn find(event: &Event, meetings: &MeetingsConfig) -> Option<Meeting> {
    for property in LINK_PROPERTIES {
        if let Some(url) = event.property_value(property).and_then(first_url) {
            return Some(with_provider(url, meetings));
        }
    }

    if let Some(url) = event.get_location().and_then(first_url) {
        return Some(with_provider(url, meetings));
    }

    let description = event.get_description()?;

    urls_in(description).find_map(|url| {
        meetings.provider_of(&url).map(|provider| Meeting {
            url,
            provider: Some(provider),
        })
    })
}

fn with_provider(url: String, meetings: &MeetingsConfig) -> Meeting {
    Meeting {
        provider: meetings.provider_of(&url),
        url,
    }
}

fn first_url(text: &str) -> Option<String> {
    urls_in(text).next()
}

fn urls_in(text: &str) -> impl Iterator<Item = String> + '_ {
    text.match_indices("http").filter_map(move |(start, _)| {
        let rest = &text[start..];

        if !rest.starts_with("http://") && !rest.starts_with("https://") {
            return None;
        }

        let end = rest
            .find(|character: char| {
                character.is_whitespace() || character == '\\' || character == '"'
            })
            .unwrap_or(rest.len());
        let url = rest[..end].trim_end_matches(['.', ',', ')', ';', '>']);

        (url.len() > "https://".len()).then(|| url.to_string())
    })
}

#[cfg(test)]
mod tests {
    use icalendar::Calendar;

    use super::*;

    fn event(body: &str) -> Event {
        let calendar: Calendar = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
             BEGIN:VEVENT\r\nUID:one\r\nSUMMARY:Test\r\n{body}END:VEVENT\r\nEND:VCALENDAR\r\n"
        )
        .parse()
        .unwrap();
        // Bound rather than returned inline: `events()` borrows `calendar`, so the clone has to
        // finish before it drops at the end of this function.
        let event = calendar.events().next().unwrap().clone();

        event
    }

    fn meetings() -> MeetingsConfig {
        MeetingsConfig::default()
    }

    #[test]
    fn a_conference_property_is_read_first() {
        let event = event(
            "LOCATION:Room 4\r\nX-GOOGLE-CONFERENCE:https://meet.google.com/jwc-vnyk-izt\r\n",
        );

        let meeting = find(&event, &meetings()).unwrap();

        assert_eq!(meeting.url, "https://meet.google.com/jwc-vnyk-izt");
        assert_eq!(meeting.provider.as_deref(), Some("meet"));
    }

    #[test]
    fn a_location_holding_a_url_is_the_meeting() {
        let event = event("LOCATION:https://chainsafe-io.zoom.us/j/81891522989\r\n");

        assert_eq!(
            find(&event, &meetings()).unwrap().provider.as_deref(),
            Some("zoom")
        );
    }

    #[test]
    fn a_location_holding_prose_is_not_a_meeting() {
        let event = event("LOCATION:Find Zoom link on ECH Discord\r\n");

        assert_eq!(find(&event, &meetings()), None);
    }

    #[test]
    fn a_description_link_counts_only_when_a_provider_claims_it() {
        let event = event(
            "DESCRIPTION:Issue: https://github.com/ethereum/pm/issues/2196\\n\\n\
             Meeting: https://ethereumfoundation.zoom.us/j/86111351882\r\n",
        );

        let meeting = find(&event, &meetings()).unwrap();

        assert_eq!(
            meeting.url,
            "https://ethereumfoundation.zoom.us/j/86111351882"
        );
        assert_eq!(meeting.provider.as_deref(), Some("zoom"));
    }

    /// Not every producer keeps `DESCRIPTION` free of markup, and a url that swallowed the
    /// closing quote reaches no provider and opens nothing.
    #[test]
    fn a_url_in_markup_ends_at_the_quote() {
        let event = event("DESCRIPTION:<a href=\"https://meet.jit.si/Room\">join</a>\r\n");

        assert_eq!(
            find(&event, &meetings()).unwrap().url,
            "https://meet.jit.si/Room"
        );
    }

    #[test]
    fn an_event_with_no_link_has_no_meeting() {
        assert_eq!(find(&event("LOCATION:Room 4\r\n"), &meetings()), None);
    }

    #[test]
    fn an_unrecognised_location_url_keeps_its_link_without_a_provider() {
        let event = event("LOCATION:https://talk.example.org/room/1\r\n");
        let meeting = find(&event, &meetings()).unwrap();

        assert_eq!(meeting.url, "https://talk.example.org/room/1");
        assert_eq!(meeting.provider, None);
    }
}
