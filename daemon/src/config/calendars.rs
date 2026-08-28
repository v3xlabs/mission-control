use serde::{Deserialize, Serialize};

use super::{version, Calendar, HumanDuration};

/// `calendars.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarsDocument {
    #[serde(default = "version::current")]
    pub version: u32,
    /// How often the daemon reconciles the rail against the feeds it has already parsed, which is
    /// separate from a feed's `refresh` and does not go back to the network.
    #[serde(default = "default_poll")]
    pub poll: HumanDuration,
    #[serde(default)]
    pub calendars: Vec<Calendar>,
}

fn default_poll() -> HumanDuration {
    HumanDuration(std::time::Duration::from_secs(60))
}

impl Default for CalendarsDocument {
    fn default() -> Self {
        Self {
            version: version::CURRENT,
            poll: default_poll(),
            calendars: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecretRef;

    #[test]
    fn a_feed_reads_its_address_from_a_file() {
        let document: CalendarsDocument = toml::from_str(
            r#"
version = 1

[[calendars]]
calendar_id = "personal"
name = "Personal"

[calendars.url]
file = "/run/secrets/personal-ics"
"#,
        )
        .unwrap();

        assert!(matches!(
            &document.calendars[0].url,
            SecretRef::File { file } if file == "/run/secrets/personal-ics"
        ));
    }

    #[test]
    fn the_defaults_cover_a_feed_that_says_only_where_it_is() {
        let document: CalendarsDocument = toml::from_str(
            r#"
[[calendars]]
calendar_id = "personal"
url = "https://example.com/basic.ics"
"#,
        )
        .unwrap();

        let calendar = &document.calendars[0];

        assert_eq!(calendar.window.seconds(), 12 * 60 * 60);
        assert_eq!(calendar.refresh.seconds(), 15 * 60);
        assert_eq!(
            calendar
                .leads
                .iter()
                .map(|lead| lead.seconds())
                .collect::<Vec<_>>(),
            [300, 0]
        );
    }

    /// Copied verbatim out of the store path the NixOS module builds: `toml.generate` sorts keys
    /// and puts every table last, so it does not write what a person would.
    #[test]
    fn the_document_the_nixos_module_generates_parses() {
        let document: CalendarsDocument = toml::from_str(
            r#"poll = "1m"
version = 1

[[calendars]]
calendar_id = "work"
leads = ["5m", "0s"]
name = "Work"
window = "12h"

[calendars.url]
file = "/run/secrets/work-ics"
"#,
        )
        .unwrap();

        let calendar = &document.calendars[0];

        assert_eq!(document.poll.seconds(), 60);
        assert_eq!(calendar.display_name(), "Work");
        assert_eq!(calendar.window.seconds(), 12 * 60 * 60);
        assert_eq!(
            calendar
                .leads
                .iter()
                .map(|lead| lead.seconds())
                .collect::<Vec<_>>(),
            [300, 0]
        );
        assert!(matches!(
            &calendar.url,
            SecretRef::File { file } if file == "/run/secrets/work-ics"
        ));
    }

    #[test]
    fn a_feed_survives_a_round_trip() {
        let document: CalendarsDocument = toml::from_str(
            r#"
[[calendars]]
calendar_id = "personal"
url = "https://example.com/basic.ics"
"#,
        )
        .unwrap();

        let body = toml::to_string(&document).unwrap();
        let reparsed: CalendarsDocument = toml::from_str(&body).unwrap();

        assert_eq!(reparsed.calendars[0].calendar_id, "personal");
    }
}
