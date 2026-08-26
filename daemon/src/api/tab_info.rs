use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{
    chrome::ChromeController,
    config::{Source, Tab},
};

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
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
