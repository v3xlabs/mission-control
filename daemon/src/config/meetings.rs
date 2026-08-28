use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MeetingsConfig {
    /// Host patterns keyed by provider name.
    #[serde(default)]
    pub providers: BTreeMap<String, Vec<String>>,
}

fn defaults() -> BTreeMap<String, Vec<String>> {
    [
        ("zoom", ["zoom.us", "*.zoom.us"].as_slice()),
        ("meet", ["meet.google.com"].as_slice()),
        ("jitsi", ["meet.jit.si", "*.jit.si"].as_slice()),
        ("webex", ["webex.com", "*.webex.com"].as_slice()),
        (
            "teams",
            ["teams.microsoft.com", "teams.live.com"].as_slice(),
        ),
    ]
    .into_iter()
    .map(|(name, hosts)| {
        (
            name.to_string(),
            hosts.iter().map(|host| (*host).to_string()).collect(),
        )
    })
    .collect()
}

impl MeetingsConfig {
    pub fn resolved(&self) -> BTreeMap<String, Vec<String>> {
        let mut providers = defaults();

        for (name, hosts) in &self.providers {
            providers.insert(name.clone(), hosts.clone());
        }

        providers
    }

    pub fn provider_of(&self, url: &str) -> Option<String> {
        let host = host_of(url)?;

        self.resolved()
            .into_iter()
            .find(|(_, patterns)| patterns.iter().any(|pattern| matches(pattern, &host)))
            .map(|(name, _)| name)
    }
}

fn host_of(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .filter(|authority| !authority.is_empty())?;
    let host = authority.rsplit('@').next()?;
    let host = match host.rsplit_once(':') {
        Some((before, port)) if port.chars().all(|character| character.is_ascii_digit()) => before,
        _ => host,
    };

    Some(host.trim_end_matches('.').to_ascii_lowercase())
}

fn matches(pattern: &str, host: &str) -> bool {
    match pattern.strip_prefix("*.") {
        Some(suffix) => host
            .strip_suffix(suffix)
            .and_then(|subdomain| subdomain.strip_suffix('.'))
            .is_some_and(|subdomain| !subdomain.is_empty()),
        None => host == pattern.to_ascii_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> MeetingsConfig {
        MeetingsConfig::default()
    }

    #[test]
    fn the_hosts_a_real_feed_uses_are_recognised() {
        let config = config();

        for (url, expected) in [
            ("https://us02web.zoom.us/j/81891522989", "zoom"),
            ("https://chainsafe-io.zoom.us/j/8189152298", "zoom"),
            ("https://zoom.us/j/123", "zoom"),
            ("https://meet.google.com/jwc-vnyk-izt", "meet"),
            ("https://meet.jit.si/SomeRoom", "jitsi"),
        ] {
            assert_eq!(config.provider_of(url).as_deref(), Some(expected), "{url}");
        }
    }

    #[test]
    fn a_url_no_pattern_claims_has_no_provider() {
        assert_eq!(config().provider_of("https://github.com/ethereum/pm"), None);
    }

    #[test]
    fn a_wildcard_does_not_claim_a_lookalike_domain() {
        assert_eq!(config().provider_of("https://evilzoom.us/j/1"), None);
    }

    #[test]
    fn a_configured_provider_replaces_only_the_one_it_names() {
        let config = MeetingsConfig {
            providers: [("jitsi".to_string(), vec!["eu.meet.example.com".to_string()])]
                .into_iter()
                .collect(),
        };

        assert_eq!(
            config
                .provider_of("https://eu.meet.example.com/MeetingId")
                .as_deref(),
            Some("jitsi")
        );
        assert_eq!(
            config.provider_of("https://us02web.zoom.us/j/1").as_deref(),
            Some("zoom"),
            "the defaults for other providers still apply"
        );
        assert_eq!(config.provider_of("https://meet.jit.si/Room"), None);
    }

    #[test]
    fn an_empty_list_turns_a_provider_off() {
        let config = MeetingsConfig {
            providers: [("zoom".to_string(), Vec::new())].into_iter().collect(),
        };

        assert_eq!(config.provider_of("https://zoom.us/j/1"), None);
    }

    #[test]
    fn a_host_is_read_without_its_port_or_credentials() {
        assert_eq!(
            host_of("https://meet.jit.si:8443/Room").as_deref(),
            Some("meet.jit.si")
        );
        assert_eq!(
            host_of("https://user:pw@MEET.JIT.SI/Room").as_deref(),
            Some("meet.jit.si")
        );
        assert_eq!(host_of("not a url"), None);
    }
}
