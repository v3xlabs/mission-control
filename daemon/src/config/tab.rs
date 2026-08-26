use serde::{Deserialize, Serialize};

use super::Source;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tab {
    pub tab_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the page stays loaded when the playlist moves on.
    #[serde(default = "persist_by_default")]
    pub persist: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    /// A stinger played while this tab loads, by name from `notifications.toml`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stinger: Option<String>,
    // Last, because a flattened source may serialise as a table and TOML puts every table after
    // the plain values of its parent.
    #[serde(flatten)]
    pub source: Source,
}

const fn persist_by_default() -> bool {
    true
}

impl Tab {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.tab_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SecretRef, TabsDocument};

    fn parse(body: &str) -> TabsDocument {
        toml::from_str(body).unwrap()
    }

    #[test]
    fn a_page_tab_reads_its_url() {
        let document = parse(
            r#"
version = 1

[[tabs]]
tab_id = "overview"
url = "https://example.com/overview"
"#,
        );

        assert!(matches!(&document.tabs[0].source, Source::Url(url) if url == "https://example.com/overview"));
    }

    #[test]
    fn a_camera_reads_its_stream_from_a_file() {
        let document = parse(
            r#"
version = 1

[[tabs]]
tab_id = "front-door"
stinger = "doorbell"

[tabs.rtsp]
file = "/run/secrets/front-door"
"#,
        );

        assert!(matches!(
            &document.tabs[0].source,
            Source::Rtsp(SecretRef::File { file }) if file == "/run/secrets/front-door"
        ));
    }

    /// TOML puts every table after the plain values of its parent, so a camera round trip is what
    /// proves the source can be written back at all.
    #[test]
    fn a_camera_survives_a_round_trip() {
        let tab = Tab {
            tab_id: "front-door".to_string(),
            name: Some("Front door".to_string()),
            persist: false,
            scale: None,
            stinger: Some("doorbell".to_string()),
            source: Source::Rtsp(SecretRef::File {
                file: "/run/secrets/front-door".to_string(),
            }),
        };

        let document = TabsDocument {
            version: 1,
            tabs: vec![tab],
        };

        let body = toml::to_string(&document).unwrap();

        assert!(matches!(
            &parse(&body).tabs[0].source,
            Source::Rtsp(SecretRef::File { file }) if file == "/run/secrets/front-door"
        ));
    }

    #[test]
    fn a_stream_url_is_never_described() {
        let source = Source::Rtsp(SecretRef::Inline("rtsp://user:pass@camera/stream".to_string()));

        assert_eq!(source.describe(), "rtsp");
    }
}
