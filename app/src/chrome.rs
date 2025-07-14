use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    pin_mut, FutureExt,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::info;

use anyhow::Result;
use async_std::{
    stream::StreamExt,
    sync::Mutex,
    task::{self, sleep},
};
use chromiumoxide::{
    cdp::browser_protocol::page::{
        EventScreencastFrame, NavigateParams, ScreencastFrameAckParams, StartScreencastFormat,
        StartScreencastParams,
    },
    Browser, BrowserConfig, Page,
};

use crate::{
    config::ChromiumConfig,
    state::{AppState, State},
};

pub struct ChromeController {
    pub current_playlist: Arc<Mutex<Option<String>>>,
    pub should_screen_capture: Arc<Mutex<bool>>,
    pub last_frame: Arc<Mutex<HashMap<String, Vec<u8>>>>,

    pub interrupt_sender: Arc<Mutex<Sender<()>>>,
    pub interrupt_receiver: Arc<Mutex<Receiver<()>>>,
    pub pages: Arc<Mutex<HashMap<String, Page>>>,
    pub browser: Arc<Mutex<Option<Arc<Browser>>>>,
}

impl Default for ChromeController {
    fn default() -> Self {
        let (interrupt_sender, interrupt_receiver) = channel(1);

        Self {
            current_playlist: Arc::new(Mutex::new(None)),
            should_screen_capture: Arc::new(Mutex::new(true)),
            last_frame: Arc::new(Mutex::new(HashMap::new())),
            interrupt_sender: Arc::new(Mutex::new(interrupt_sender)),
            interrupt_receiver: Arc::new(Mutex::new(interrupt_receiver)),
            pages: Arc::new(Mutex::new(HashMap::new())),
            browser: Arc::new(Mutex::new(None)),
        }
    }
}

impl ChromeController {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(&self, config: &ChromiumConfig, state: &State) -> Result<()> {
        let home_path = std::env::var("HOME").unwrap();
        let user_data_dir = format!("{}/.chromium-mission-data", home_path);
        let browser_config = BrowserConfig::builder();

        let mut browser_config = browser_config
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
            .arg("--remote-debugging-address=0.0.0.0")
            .port(9222)
            .arg("--no-first-run")
            .arg("--password-store=basic")
            .viewport(None);

        let theme = config.theme.as_deref().unwrap_or("dark");

        if theme == "light" {
            browser_config = browser_config.arg(" --disable-features=WebContentsForceDark");
        } else if theme == "dark" {
            browser_config = browser_config.arg(" --enable-features=WebContentsForceDark:inversion_method/cielab_based/image_behavior/selective/text_lightness_threshold/150/background_lightness_threshold/205");
        }

        let browser_config = browser_config.build().unwrap();

        info!("{:?}", browser_config);

        let (browser_raw, mut handler) = Browser::launch(browser_config).await?;
        let browser = std::sync::Arc::new(browser_raw);

        // store browser for later external actions
        {
            let mut br_lock = self.browser.lock().await;
            *br_lock = Some(browser.clone());
        }

        task::spawn(async move {
            while let Some(event) = handler.next().await {
                info!("Event: {:?}", event);
            }
        });

        sleep(Duration::from_secs(2)).await;

        // Open all tabs that should be persisted
        let tabs = config.tabs.clone();

        if let Some(tabs) = tabs {
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

                page.execute(
                    StartScreencastParams::builder()
                        .format(StartScreencastFormat::Jpeg)
                        .quality(80)
                        .build(),
                )
                .await?;

                let page_ref = page.clone();
                let self_arc = self.last_frame.clone();
                let key_clone = key.clone();

                task::spawn(async move {
                    let mut events = page_ref
                        .event_listener::<EventScreencastFrame>()
                        .await
                        .unwrap();
                    while let Some(frame) = events.next().await {
                        // info!("Event: {:?}", frame);

                        let frame_buf: &[u8] = frame.data.as_ref();
                        info!("Received frame: {}", frame_buf.len());

                        self_arc
                            .lock()
                            .await
                            .insert(key_clone.clone(), frame_buf.to_vec());

                        // Acknowledge the frame to continue the stream
                        page_ref
                            .execute(
                                ScreencastFrameAckParams::builder()
                                    .session_id(frame.session_id)
                                    .build()
                                    .unwrap(),
                            )
                            .await
                            .unwrap();
                    }
                });

                self.pages.lock().await.insert(key.clone(), page);
            }
        }

        if let Some(playlists) = &config.playlists {
            if let Some((playlist_name, _playlist_config)) = playlists.iter().next() {
                // this is now the default playlist
                self.current_playlist
                    .lock()
                    .await
                    .replace(playlist_name.clone());
            }
        }

        self.run(config, state, &browser).await;

        Ok(())
    }

    pub async fn run(&self, config: &ChromiumConfig, state: &Arc<AppState>, browser: &Arc<Browser>) {
        let mut current_tab = 0;
        loop {
            let mut sleep_duration = Duration::from_secs(10000);

            if let Some(playlists) = &config.playlists {
                if let Some(playlist) = self.current_playlist.lock().await.as_ref() {
                    state
                        .hass
                        .playlist_entity
                        .update_state(&state.hass.mqtt_client, playlist);

                    if let Some(playlist_config) = playlists.get(playlist) {
                        // TODO: implement playlist logic
                        if let Some(tab) = playlist_config.tabs.get(current_tab) {
                            if let Some(page) = self.pages.lock().await.get(tab) {
                                info!("Activated tab \"{}\"", tab);
                                page.activate().await.unwrap();

                                state
                                    .hass
                                    .tab_entity
                                    .update_state(&state.hass.mqtt_client, tab);

                                state.hass.url_entity.update_state(
                                    &state.hass.mqtt_client,
                                    &config.tabs.as_ref().unwrap().get(tab).unwrap().url,
                                );
                            } else {
                                info!("On demand activating tab \"{}\"", tab);
                                if let Some(tab_config) = config.tabs.as_ref().unwrap().get(tab) {
                                    let page = browser.new_page("about:blank").await.unwrap();
                                    page.execute(
                                        NavigateParams::builder()
                                            .url(tab_config.url.clone())
                                            .build()
                                            .unwrap(),
                                    )
                                    .await
                                    .unwrap();

                                    // TODO: update tab entity
                                    state
                                        .hass
                                        .tab_entity
                                        .update_state(&state.hass.mqtt_client, tab);

                                    // TODO: update url entity
                                    state
                                        .hass
                                        .url_entity
                                        .update_state(&state.hass.mqtt_client, &tab_config.url);
                                } else {
                                    info!("Tab \"{}\" config not found", tab);
                                    current_tab = 0;
                                    continue;
                                }
                            }
                        } else {
                            info!(
                                "Tab \"{}\" not found in playlist \"{}\"",
                                current_tab, playlist
                            );
                            current_tab = 0;
                            continue;
                        }

                        // sleep(Duration::from_secs(playlist_config.interval as u64)).await;
                        sleep_duration = Duration::from_secs(playlist_config.interval as u64);
                        current_tab += 1;
                        if current_tab >= playlist_config.tabs.len() {
                            current_tab = 0;
                        }
                    } else {
                        info!("Playlist \"{}\" config not found", playlist);
                        sleep_duration = Duration::from_secs(10000);
                    }
                } else {
                    info!("No playlist selected");
                    sleep_duration = Duration::from_secs(10000);
                }
            }

            if sleep_duration == Duration::from_secs(0) {
                info!("No sleep duration, continuing loop");
                continue;
            }

            let sleep_future = sleep(sleep_duration).fuse();
            use futures::pin_mut;

            pin_mut!(sleep_future);
            let mut interrupt_receiver = self.interrupt_receiver.lock().await;
            let interrupt_future = interrupt_receiver.next().fuse();
            pin_mut!(interrupt_future);

            futures::select! {
                _ = sleep_future => {
                    // Sleep completed normally, continue loop
                    info!("Sleep completed normally, continuing loop");
                    continue;
                }
                _ = interrupt_future => {
                    // Received interrupt signal, break the loop
                    info!("Received interrupt signal, stopping playlist");
                    continue;
                }
            }
        }
    }

    pub async fn set_playlist(&self, playlist: String) {
        self.current_playlist.lock().await.replace(playlist);
    }

    /// Activate a specific tab immediately, opening it if necessary.
    pub async fn activate_tab(&self, tab_id: &str, state: &Arc<AppState>) -> Result<()> {
        let config = match &state.config.chromium {
            Some(c) => c,
            None => return Ok(()),
        };

        let mut pages = self.pages.lock().await;
        if let Some(page) = pages.get(tab_id) {
            page.activate().await?;
        } else {
            // need to open
            let browser_opt = self.browser.lock().await;
            if let Some(browser) = browser_opt.as_ref() {
                if let Some(tab_config) = config.tabs.as_ref().and_then(|t| t.get(tab_id)) {
                    let page = browser.new_page("about:blank").await?;
                    page.execute(
                        NavigateParams::builder()
                            .url(tab_config.url.clone())
                            .build()
                            .unwrap(),
                    )
                    .await?;
                    pages.insert(tab_id.to_string(), page.clone());
                    page.activate().await?;
                }
            }
        }

        // update hass entities
        state.hass.tab_entity.update_state(&state.hass.mqtt_client, tab_id);
        if let Some(tab_config) = config.tabs.as_ref().and_then(|t| t.get(tab_id)) {
            state
                .hass
                .url_entity
                .update_state(&state.hass.mqtt_client, &tab_config.url);
        }

        Ok(())
    }
}
