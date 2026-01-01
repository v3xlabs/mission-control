use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use async_std::{sync::Mutex, task};
use chromiumoxide::{
    cdp::browser_protocol::page::{
        AddScriptToEvaluateOnNewDocumentParams, EventScreencastFrame, NavigateParams,
        ScreencastFrameAckParams, StartScreencastFormat, StartScreencastParams,
    },
    Browser, BrowserConfig, Page,
};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};

use crate::{
    config::ChromiumConfig,
    db::models::TabWithOrder,
    db::repositories::{PlaylistRepository, PlaylistTabRepository, TabRepository},
    state::AppState,
};

use super::{ChromeMessage, ChromeResponse, ChromeState};

const SCREENCAST_MAX_FPS: u64 = 4;
const SCREENCAST_MAX_BYTES: usize = 5_000_000;

pub struct ChromeController {
    pub state: Arc<Mutex<ChromeState>>,
    browser: Arc<Mutex<Option<Arc<Browser>>>>,
    pages: Arc<Mutex<HashMap<String, Arc<Page>>>>,
    pub last_frame: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    viewport: Arc<Mutex<HashMap<String, (i32, i32)>>>,
    should_screen_capture: Arc<Mutex<bool>>,
    auto_task: Arc<Mutex<Option<task::JoinHandle<()>>>>,
    message_sender: Sender<ChromeMessage>,
    message_receiver: Arc<Mutex<Receiver<ChromeMessage>>>,
    response_sender: Arc<Mutex<Option<Sender<ChromeResponse>>>>,
}

impl Default for ChromeController {
    fn default() -> Self {
        Self::new()
    }
}

impl ChromeController {
    pub fn new() -> Self {
        let (message_sender, message_receiver) = channel(100);
        Self {
            state: Arc::new(Mutex::new(ChromeState::default())),
            browser: Arc::new(Mutex::new(None)),
            pages: Arc::new(Mutex::new(HashMap::new())),
            last_frame: Arc::new(Mutex::new(HashMap::new())),
            viewport: Arc::new(Mutex::new(HashMap::new())),
            should_screen_capture: Arc::new(Mutex::new(true)),
            auto_task: Arc::new(Mutex::new(None)),
            message_sender,
            message_receiver: Arc::new(Mutex::new(message_receiver)),
            response_sender: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_message_sender(&self) -> Sender<ChromeMessage> {
        self.message_sender.clone()
    }

    pub async fn start(
        self: &Arc<Self>,
        config: &ChromiumConfig,
        app_state: &Arc<AppState>,
    ) -> Result<()> {
        if self.browser.lock().await.is_none() {
            self.launch_browser(config).await?;
        }
        let controller = Arc::clone(self);
        let app_state_clone = app_state.clone();
        let config_clone = config.clone();
        task::spawn(async move {
            controller
                .run_message_loop(app_state_clone, config_clone)
                .await
        });
        self.ensure_active_playlist(app_state).await?;
        Ok(())
    }

    fn build_browser_config(config: &ChromiumConfig) -> Result<BrowserConfig> {
        BrowserConfig::builder()
            .chrome_executable(
                config
                    .binary_path
                    .clone()
                    .unwrap_or_else(|| "chromium".to_string()),
            )
            .with_head()
            .disable_default_args()
            // baseline defaults from chromiumoxide minus --enable-automation
            .arg("--disable-background-networking")
            .arg("--enable-features=NetworkService,NetworkServiceInProcess")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-breakpad")
            .arg("--disable-client-side-phishing-detection")
            .arg("--disable-component-extensions-with-background-pages")
            .arg("--disable-default-apps")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-extensions")
            .arg("--disable-features=TranslateUI")
            .arg("--disable-hang-monitor")
            .arg("--disable-ipc-flooding-protection")
            .arg("--disable-popup-blocking")
            .arg("--disable-prompt-on-repost")
            .arg("--disable-renderer-backgrounding")
            .arg("--disable-sync")
            .arg("--force-color-profile=srgb")
            .arg("--metrics-recording-only")
            .arg("--no-first-run")
            .arg("--password-store=basic")
            .arg("--use-mock-keychain")
            .arg("--enable-blink-features=IdleDetection")
            .arg("--lang=en_US")
            // kiosk/automation-hiding
            .arg("--no-sandbox")
            .arg("--disable-gpu")
            .arg("--kiosk")
            .arg("--disable-infobars")
            .arg("--disable-automation")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-session-crashed-bubble")
            .viewport(None)
            .build()
            .map_err(|e| anyhow!("Failed to build browser config: {}", e))
    }

    async fn launch_browser(&self, config: &ChromiumConfig) -> Result<()> {
        let cfg = Self::build_browser_config(config)?;
        let (browser, mut handler) = Browser::launch(cfg).await?;
        // Keep the handler alive in a background task
        task::spawn(async move {
            while let Some(evt) = handler.next().await {
                if let Err(e) = evt {
                    error!("Chromium handler error: {:?}", e);
                    continue;
                }
            }
            warn!("Chromium handler loop ended");
        });
        *self.browser.lock().await = Some(Arc::new(browser));
        info!("Chromium launched");
        Ok(())
    }

    async fn run_message_loop(self: Arc<Self>, app_state: Arc<AppState>, config: ChromiumConfig) {
        while let Some(msg) = { self.message_receiver.lock().await.next().await } {
            info!("chrome message loop received: {:?}", msg);
            let resp = match self.handle_message(msg, &app_state, &config).await {
                Ok(r) => r,
                Err(e) => ChromeResponse::Error {
                    message: e.to_string(),
                },
            };
            if let Some(sender) = &mut *self.response_sender.lock().await {
                let _ = sender.send(resp).await;
            }
        }
        error!("Chrome message loop exited");
    }

    async fn handle_message(
        &self,
        msg: ChromeMessage,
        app_state: &Arc<AppState>,
        _config: &ChromiumConfig,
    ) -> Result<ChromeResponse> {
        match msg {
            ChromeMessage::ActivatePlaylist { playlist_id } => {
                self.activate_playlist(playlist_id, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::ActivateTab {
                tab_id,
                playlist_id,
            } => {
                self.activate_tab(tab_id, playlist_id, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::NextTab => {
                self.next_tab(app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::PreviousTab => {
                self.previous_tab(app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::UpdateTabUrl { tab_id, url } => {
                self.update_tab_url(tab_id, url, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::CloseTab { tab_id } => {
                self.close_tab(tab_id).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::RefreshTab { tab_id } => {
                self.refresh_tab(tab_id).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::RecreateTab { tab_id } => {
                self.recreate_tab(tab_id, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::UpdateInterval {
                playlist_id,
                interval_seconds,
            } => {
                app_state
                    .playlist_repository
                    .update_interval(&playlist_id, interval_seconds)
                    .await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::ReloadTab => {
                self.reload_current_tab().await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::GetStatus => {
                let st = self.state.lock().await;
                Ok(ChromeResponse::Status {
                    current_playlist_id: st.current_playlist_id.clone(),
                    current_tab_id: st.current_tab_id.clone(),
                    is_running: st.is_running,
                    auto_rotate: st.auto_rotate,
                })
            }
            ChromeMessage::StopPlaylist => {
                self.stop_auto_rotation().await;
                let mut st = self.state.lock().await;
                st.is_running = false;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::StartPlaylist => {
                if let Some(pid) = self.state.lock().await.current_playlist_id.clone() {
                    if let Some(pl) = app_state.playlist_repository.get_by_id(&pid).await? {
                        self.start_auto_rotation(pl.interval_seconds).await?;
                    }
                }
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::Shutdown => {
                self.shutdown().await?;
                Ok(ChromeResponse::Success)
            }
        }
    }

    async fn ensure_active_playlist(&self, app_state: &Arc<AppState>) -> Result<()> {
        let playlists = app_state.playlist_repository.get_all().await?;
        if playlists.is_empty() {
            return Ok(());
        }
        if let Some(active) = playlists.iter().find(|p| p.is_active) {
            self.activate_playlist(active.id.clone(), app_state).await?;
            return Ok(());
        }
        let first = playlists.first().unwrap();
        app_state
            .playlist_repository
            .set_active(&first.id, true)
            .await?;
        self.activate_playlist(first.id.clone(), app_state).await
    }

    async fn get_enabled_tabs(
        app_state: &Arc<AppState>,
        playlist_id: &str,
    ) -> Result<Vec<TabWithOrder>> {
        let mut tabs = app_state
            .playlist_tab_repository
            .get_playlist_tabs(playlist_id)
            .await?;
        tabs.retain(|t| t.enabled);
        tabs.sort_by_key(|t| t.order_index);
        Ok(tabs)
    }

    async fn activate_playlist(
        &self,
        playlist_id: String,
        app_state: &Arc<AppState>,
    ) -> Result<()> {
        info!("activate_playlist: {}", &playlist_id);
        let _playlist = app_state
            .playlist_repository
            .get_by_id(&playlist_id)
            .await?
            .ok_or_else(|| anyhow!("Playlist {} not found", playlist_id))?;
        let tabs = Self::get_enabled_tabs(app_state, &playlist_id).await?;
        if tabs.is_empty() {
            warn!("Playlist {} has no tabs", playlist_id);
            return Ok(());
        }
        info!(
            "activate_playlist {} tabs: {:?}",
            playlist_id,
            tabs.iter().map(|t| t.id.clone()).collect::<Vec<_>>()
        );

        let all = app_state.playlist_repository.get_all().await?;
        for p in all {
            app_state
                .playlist_repository
                .set_active(&p.id, p.id == playlist_id)
                .await?;
        }
        let playlist_ids: Vec<String> = app_state
            .playlist_repository
            .get_all()
            .await?
            .into_iter()
            .map(|p| p.id)
            .collect();
        app_state
            .hass
            .publish_playlist_options(playlist_ids, Some(&playlist_id));

        {
            let mut st = self.state.lock().await;
            st.current_playlist_id = Some(playlist_id.clone());
            st.current_tab_index = 0;
            st.is_running = true;
            st.auto_rotate = false;
        }

        for tab in &tabs {
            if tab.persist && !self.pages.lock().await.contains_key(&tab.id) {
                info!("preloading persistent tab {}", tab.id);
                if let Err(e) = self.create_tab_page(&tab.id, &tab.url, app_state).await {
                    warn!("failed to preload tab {}: {}", tab.id, e);
                }
            }
        }

        app_state.hass.publish_tab_options(&tabs, Some(&tabs[0].id));

        if let Some(pl) = app_state
            .playlist_repository
            .get_by_id(&playlist_id)
            .await?
        {
            self.start_auto_rotation(pl.interval_seconds).await?;
        }

        self.activate_tab(tabs[0].id.clone(), playlist_id, app_state)
            .await?;
        Ok(())
    }

    async fn activate_tab(
        &self,
        tab_id: String,
        playlist_id: String,
        app_state: &Arc<AppState>,
    ) -> Result<()> {
        let tab = app_state
            .tab_repository
            .get_by_id(&tab_id)
            .await?
            .ok_or_else(|| anyhow!("Tab {} not found", tab_id))?;

        let tabs_for_playlist = Self::get_enabled_tabs(app_state, &playlist_id).await?;
        let has_page = self.pages.lock().await.contains_key(&tab_id);
        info!(
            "activate_tab {} in playlist {}, has_page={}, url={}",
            tab_id, playlist_id, has_page, tab.url
        );
        if !has_page {
            self.create_tab_page(&tab_id, &tab.url, app_state).await?;
        }

        let page = {
            let pages = self.pages.lock().await;
            pages
                .get(&tab_id)
                .cloned()
                .ok_or_else(|| anyhow!("No page for tab {}", tab_id))?
        };
        page.bring_to_front().await?;

        {
            let mut st = self.state.lock().await;
            st.current_tab_id = Some(tab_id.clone());
            st.current_playlist_id = Some(playlist_id.clone());
            st.current_tab_opened_at = Some(std::time::SystemTime::now());
        }

        app_state
            .hass
            .publish_tab_options(&tabs_for_playlist, Some(&tab_id));
        app_state
            .hass
            .url_entity
            .update_state(&app_state.hass.mqtt_client, &tab.url);

        Ok(())
    }

    async fn create_tab_page(
        &self,
        tab_id: &str,
        url: &str,
        app_state: &Arc<AppState>,
    ) -> Result<()> {
        let browser = {
            let b = self.browser.lock().await;
            b.clone().ok_or_else(|| anyhow!("Browser not ready"))?
        };
        let page = browser.new_page(url).await?;
        // Suppress automation banner / webdriver detection
        let _ = page
            .execute(
                AddScriptToEvaluateOnNewDocumentParams::builder()
                    .source(
                        "Object.defineProperty(navigator, 'webdriver', { get: () => undefined });",
                    )
                    .build()
                    .unwrap(),
            )
            .await;
        let page_arc = Arc::new(page);
        self.pages
            .lock()
            .await
            .insert(tab_id.to_string(), page_arc.clone());
        info!("create_tab_page: stored page for {}", tab_id);

        if *self.should_screen_capture.lock().await {
            let page_ref = page_arc.clone();
            let frames = self.last_frame.clone();
            let tab_key = tab_id.to_string();
            task::spawn(async move {
                let started = page_ref
                    .execute(
                        StartScreencastParams::builder()
                            .format(StartScreencastFormat::Jpeg)
                            .quality(80)
                            .build(),
                    )
                    .await;
                if let Err(e) = started {
                    warn!("screencast start failed for {}: {:?}", tab_key, e);
                    return;
                }
                if let Ok(mut events) = page_ref.event_listener::<EventScreencastFrame>().await {
                    let mut last = Instant::now();
                    let min_interval = Duration::from_millis(1000 / SCREENCAST_MAX_FPS);
                    while let Some(frame) = events.next().await {
                        let buf: &[u8] = frame.data.as_ref();
                        let now = Instant::now();
                        let _ = page_ref
                            .execute(
                                ScreencastFrameAckParams::builder()
                                    .session_id(frame.session_id)
                                    .build()
                                    .unwrap(),
                            )
                            .await;
                        if buf.len() > SCREENCAST_MAX_BYTES
                            || now.duration_since(last) < min_interval
                        {
                            continue;
                        }
                        last = now;
                        frames.lock().await.insert(tab_key.clone(), buf.to_vec());
                    }
                } else {
                    warn!("screencast listener failed for {}", tab_key);
                }
            });
        }

        if let (Ok(w), Ok(h)) = (
            page_arc.evaluate("window.innerWidth").await,
            page_arc.evaluate("window.innerHeight").await,
        ) {
            if let (Some(wv), Some(hv)) = (w.value(), h.value()) {
                if let (Some(wi), Some(hi)) = (wv.as_u64(), hv.as_u64()) {
                    self.viewport
                        .lock()
                        .await
                        .insert(tab_id.to_string(), (wi as i32, hi as i32));
                    let app = app_state.clone();
                    let tab = tab_id.to_string();
                    task::spawn(async move {
                        let _ = app
                            .tab_repository
                            .update_viewport_dimensions(&tab, wi as i32, hi as i32)
                            .await;
                    });
                }
            }
        }

        Ok(())
    }

    async fn next_tab(&self, app_state: &Arc<AppState>) -> Result<()> {
        let (playlist_id, current_tab_id, current_index) = {
            let st = self.state.lock().await;
            (
                st.current_playlist_id.clone(),
                st.current_tab_id.clone(),
                st.current_tab_index,
            )
        };
        let Some(pid) = playlist_id else {
            return Ok(());
        };
        let tabs = Self::get_enabled_tabs(app_state, &pid).await?;
        if tabs.is_empty() {
            return Ok(());
        }

        let idx = current_tab_id
            .as_ref()
            .and_then(|tid| tabs.iter().position(|t| &t.id == tid))
            .unwrap_or(current_index)
            % tabs.len();
        let next = (idx + 1) % tabs.len();
        info!("next_tab: playlist {}, idx {} -> {}", pid, idx, next);
        self.activate_tab(tabs[next].id.clone(), pid, app_state)
            .await?;
        {
            let mut st = self.state.lock().await;
            st.current_tab_index = next;
        }
        Ok(())
    }

    async fn previous_tab(&self, app_state: &Arc<AppState>) -> Result<()> {
        let (playlist_id, current_index) = {
            let st = self.state.lock().await;
            (st.current_playlist_id.clone(), st.current_tab_index)
        };
        let Some(pid) = playlist_id else {
            return Ok(());
        };
        let tabs = Self::get_enabled_tabs(app_state, &pid).await?;
        if tabs.is_empty() {
            return Ok(());
        }

        let prev = if current_index == 0 {
            tabs.len() - 1
        } else {
            current_index - 1
        };
        info!(
            "prev_tab: playlist {}, idx {} -> {}",
            pid, current_index, prev
        );
        self.activate_tab(tabs[prev].id.clone(), pid, app_state)
            .await?;
        {
            let mut st = self.state.lock().await;
            st.current_tab_index = prev;
        }
        Ok(())
    }

    async fn start_auto_rotation(&self, interval_seconds: i64) -> Result<()> {
        if interval_seconds <= 0 {
            return Ok(());
        }
        self.stop_auto_rotation().await;
        {
            let mut st = self.state.lock().await;
            st.auto_rotate = true;
        }
        let sender = self.message_sender.clone();
        let state = self.state.clone();
        let handle = task::spawn(async move {
            loop {
                task::sleep(Duration::from_secs(interval_seconds as u64)).await;
                if !state.lock().await.auto_rotate {
                    break;
                }
                let _ = sender.clone().send(ChromeMessage::NextTab).await;
            }
        });
        *self.auto_task.lock().await = Some(handle);
        Ok(())
    }

    async fn stop_auto_rotation(&self) {
        {
            let mut st = self.state.lock().await;
            st.auto_rotate = false;
        }
        if let Some(handle) = self.auto_task.lock().await.take() {
            let _ = handle.cancel().await;
        }
    }

    async fn reload_current_tab(&self) -> Result<()> {
        if let Some(tab) = self.state.lock().await.current_tab_id.clone() {
            if let Some(page) = self.pages.lock().await.get(&tab) {
                page.reload().await?;
            }
        }
        Ok(())
    }

    async fn update_tab_url(
        &self,
        tab_id: String,
        url: String,
        app_state: &Arc<AppState>,
    ) -> Result<()> {
        app_state.tab_repository.update_url(&tab_id, &url).await?;
        if let Some(page) = self.pages.lock().await.get(&tab_id) {
            page.execute(NavigateParams::builder().url(url.clone()).build().unwrap())
                .await?;
        }
        Ok(())
    }

    async fn close_tab(&self, tab_id: String) -> Result<()> {
        if let Some(page) = self.pages.lock().await.remove(&tab_id) {
            let _ = page.clone(); // drop to close later when browser exits
        }
        self.last_frame.lock().await.remove(&tab_id);
        self.viewport.lock().await.remove(&tab_id);
        Ok(())
    }

    async fn refresh_tab(&self, tab_id: String) -> Result<()> {
        if let Some(page) = self.pages.lock().await.get(&tab_id) {
            page.reload().await?;
        }
        Ok(())
    }

    async fn recreate_tab(&self, tab_id: String, app_state: &Arc<AppState>) -> Result<()> {
        self.close_tab(tab_id.clone()).await?;
        let tab = app_state
            .tab_repository
            .get_by_id(&tab_id)
            .await?
            .ok_or_else(|| anyhow!("Tab {} missing", tab_id))?;
        self.create_tab_page(&tab.id, &tab.url, app_state).await
    }

    async fn shutdown(&self) -> Result<()> {
        self.stop_auto_rotation().await;
        let pages = self.pages.lock().await.drain().collect::<Vec<_>>();
        for (_, page) in pages {
            let _ = page.clone();
        }
        *self.browser.lock().await = None;
        Ok(())
    }

    pub async fn get_screenshot(&self, tab_id: &str) -> Option<Vec<u8>> {
        self.last_frame.lock().await.get(tab_id).cloned()
    }

    pub async fn get_viewport_dimensions(&self, tab_id: &str) -> Option<(i32, i32)> {
        self.viewport.lock().await.get(tab_id).cloned()
    }
}
