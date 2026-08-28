use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    chrome::ChromeController,
    config::{Source, Tab},
};

// A `None` is left out rather than written as `null`: the generated client types read an optional
// field as absent, and a camera arriving as `{"url": null}` is read there as a page with an empty
// address.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[oai(skip_serializing_if_is_none)]
pub struct TabInfo {
    pub tab_id: String,
    pub name: String,
    /// The page address. Absent for a camera, whose stream url is a credential and never leaves
    /// the daemon.
    pub url: Option<String>,
    pub order_index: usize,
    pub persist: bool,
    pub enabled: bool,
    pub viewport_width: Option<i32>,
    pub viewport_height: Option<i32>,
}

impl TabInfo {
    pub async fn new(
        tab: &Tab,
        order_index: usize,
        enabled: bool,
        chrome: &ChromeController,
    ) -> Self {
        let (viewport_width, viewport_height) = chrome
            .viewport(&tab.tab_id)
            .await
            .map_or((None, None), |(width, height)| (Some(width), Some(height)));

        Self {
            tab_id: tab.tab_id.clone(),
            name: tab.display_name().to_string(),
            url: match &tab.source {
                Source::Url(url) => Some(url.clone()),
                Source::Rtsp(_) => None,
            },
            order_index,
            persist: tab.persist,
            enabled,
            viewport_width,
            viewport_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use poem_openapi::types::ToJSON;

    use super::*;
    use crate::config::SecretRef;

    /// The web UI decides between a page and a camera on whether `url` is there at all, so a
    /// camera that reports `null` gets treated as a page with an empty address.
    #[tokio::test]
    async fn a_camera_leaves_the_url_out() {
        let tab = Tab {
            tab_id: "entrance-camera".to_string(),
            name: Some("Entrance".to_string()),
            persist: false,
            scale: None,
            stinger: None,
            source: Source::Rtsp(SecretRef::Inline("rtsp://camera/stream".to_string())),
        };

        let info = TabInfo::new(&tab, 0, true, &ChromeController::new()).await;

        assert!(info.to_json().unwrap().get("url").is_none());
    }
}
