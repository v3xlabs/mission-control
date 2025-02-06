use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use async_std::{
    process::Command,
    stream::StreamExt,
    sync::Mutex,
    task::{self, sleep},
};
use chromiumoxide::{cdp::browser_protocol::page::NavigateParams, Browser, BrowserConfig, Page};

use crate::config::ChromiumConfig;

pub struct ChromeController {
    current_playlist: Mutex<Option<String>>,
}

impl ChromeController {
    pub fn new() -> Self {
        Self {
            current_playlist: Mutex::new(None),
        }
    }

    pub async fn start(&self, config: &ChromiumConfig) -> Result<()> {
        let home_path = std::env::var("HOME").unwrap();
        let user_data_dir = format!("{}/.chromium-mission-data", home_path);
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .chrome_executable(config.binary_path.clone().unwrap_or("chromium".to_string()))
                .disable_default_args()
                .with_head()
                .enable_cache()
                .user_data_dir(user_data_dir)
                .arg("--kiosk")
                .arg("--noerrdialogs")
                .arg("--disable-infobars")
                .arg("--disable-session-crashed-bubble")
                .arg("--disable-features=TranslateUI")
                .arg("--remote-debugging-port=9222")
                .arg("--no-first-run")
                .arg("--password-store=basic")
                .viewport(None)
                .build()
                .unwrap(),
        )
        .await?;

        task::spawn(async move { while let Some(_) = handler.next().await {} });

        // Open all tabs that should be persisted
        let mut pages: HashMap<String, Page> = HashMap::new();

        if let Some(tabs) = &config.tabs {
            for (key, tab) in tabs {
                // Open a new tab for user-requested URL.
                let page = browser.new_page("about:blank").await?;
                page.execute(
                    NavigateParams::builder()
                        .url(tab.url.clone())
                        .build()
                        .unwrap(),
                )
                .await
                .unwrap();

                pages.insert(key.clone(), page);
            }
        }

        if let Some(playlists) = &config.playlists {
            if let Some((playlist_name, playlist_config)) = playlists.iter().next() {
                // this is now the default playlist
                self.current_playlist
                    .lock()
                    .await
                    .replace(playlist_name.clone());
            }
        }

        let mut current_tab = 0;
        while true {
            if let Some(playlists) = &config.playlists {
                if let Some(playlist) = self.current_playlist.lock().await.as_ref() {
                    if let Some(playlist_config) = playlists.get(playlist) {
                        // TODO: implement playlist logic
                        if let Some(tab) = playlist_config.tabs.get(current_tab) {
                            if let Some(page) = pages.get(tab) {
                                println!("Activated tab \"{}\"", tab);
                                page.activate().await.unwrap();
                            } else {
                                println!("On demand activating tab \"{}\"", tab);
                                if let Some(tab_config) = config.tabs.as_ref().unwrap().get(tab) {
                                    let page = browser.new_page("about:blank").await?;
                                    page.execute(
                                        NavigateParams::builder()
                                            .url(tab_config.url.clone())
                                            .build()
                                            .unwrap(),
                                    )
                                    .await
                                    .unwrap();
                                } else {
                                    println!("Tab \"{}\" config not found", tab);
                                    current_tab = 0;
                                }
                            }
                        } else {
                            println!(
                                "Tab \"{}\" not found in playlist \"{}\"",
                                current_tab, playlist
                            );
                            current_tab = 0;
                        }

                        sleep(Duration::from_secs(playlist_config.interval as u64)).await;
                        current_tab += 1;
                        if current_tab >= playlist_config.tabs.len() {
                            current_tab = 0;
                        }
                    } else {
                        println!("Playlist \"{}\" config not found", playlist);
                        sleep(Duration::from_secs(10000)).await;
                    }
                } else {
                    println!("No playlist selected");
                    sleep(Duration::from_secs(10000)).await;
                }
            } else {
                // Wait a while before checking in again
                sleep(Duration::from_secs(10000)).await;
            }
        }

        Ok(())
    }
}
