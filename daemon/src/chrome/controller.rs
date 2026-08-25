use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use chromiumoxide::{
    cdp::browser_protocol::{
        emulation::SetDeviceMetricsOverrideParams,
        page::{NavigateParams, StopScreencastParams},
    },
    Browser, BrowserConfig, Page,
};
use futures::StreamExt;
use tokio::sync::{mpsc, watch, Mutex};
use tracing::{error, info, warn};

use crate::{
    config::{ChromiumConfig, Tab},
    state::AppState,
};

use super::{capture, ChromeMessage, ChromeResponse, ChromeState, Preview, Request};

pub struct ChromeController {
    pub state: Arc<Mutex<ChromeState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    pages: Arc<Mutex<HashMap<String, Page>>>,
    previews: Arc<Mutex<HashMap<String, Preview>>>,
    viewport: Arc<Mutex<HashMap<String, (i32, i32)>>>,
    auto_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    sender: mpsc::Sender<Request>,
    receiver: Mutex<Option<mpsc::Receiver<Request>>>,
    running: AtomicBool,
}

impl Default for ChromeController {
    fn default() -> Self {
        Self::new()
    }
}

impl ChromeController {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);

        Self {
            state: Arc::new(Mutex::new(ChromeState::default())),
            browser: Arc::new(Mutex::new(None)),
            pages: Arc::new(Mutex::new(HashMap::new())),
            previews: Arc::new(Mutex::new(HashMap::new())),
            viewport: Arc::new(Mutex::new(HashMap::new())),
            auto_task: Arc::new(Mutex::new(None)),
            sender,
            receiver: Mutex::new(Some(receiver)),
            running: AtomicBool::new(false),
        }
    }

    pub fn sender(&self) -> mpsc::Sender<Request> {
        self.sender.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub async fn start(self: &Arc<Self>, app_state: &Arc<AppState>) -> Result<()> {
        let config = app_state.config.read().await;

        if self.browser.lock().await.is_none() {
            self.launch_browser(&config.device.chromium, &app_state.config.dirs.cache)
                .await?;
        }

        let receiver = self
            .receiver
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow!("chrome controller already started"))?;

        let controller = Arc::clone(self);
        let state = app_state.clone();

        self.running.store(true, Ordering::Relaxed);
        tokio::spawn(async move { controller.run_message_loop(receiver, state).await });

        self.activate_default_playlist(app_state).await
    }

    fn build_browser_config(config: &ChromiumConfig, cache: &std::path::Path) -> Result<BrowserConfig> {
        let mut builder = BrowserConfig::builder()
            .chrome_executable(
                config
                    .binary_path
                    .clone()
                    .or_else(|| std::env::var("CHROMIUM_BINARY").ok())
                    .unwrap_or_else(|| "chromium".to_string()),
            )
            .user_data_dir(cache.join("chromium-profile"))
            .with_head()
            .disable_default_args()
            .arg("--disable-background-networking")
            .arg("--enable-features=NetworkService,NetworkServiceInProcess")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-breakpad")
            .arg("--disable-client-side-phishing-detection")
            .arg("--disable-component-extensions-with-background-pages")
            .arg("--disable-default-apps")
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
            .arg("--lang=en_US")
            .arg("--ozone-platform=wayland")
            .arg("--disable-infobars")
            .arg("--disable-session-crashed-bubble")
            .viewport(None);

        // A full-screen window is not subject to the space a layer surface reserves, so the
        // overlay sidebar needs a window the compositor can tile.
        if config.fullscreen {
            builder = builder.arg("--kiosk");
        } else {
            builder = builder.arg("--start-maximized");
        }

        for arg in &config.extra_args {
            builder = builder.arg(arg.as_str());
        }

        builder
            .build()
            .map_err(|error| anyhow!("failed to build browser config: {error}"))
    }

    async fn launch_browser(&self, config: &ChromiumConfig, cache: &std::path::Path) -> Result<()> {
        let (browser, mut handler) = Browser::launch(Self::build_browser_config(config, cache)?).await?;

        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(error) = event {
                    error!("chromium handler error: {error:?}");
                }
            }
            warn!("chromium handler loop ended");
        });

        *self.browser.lock().await = Some(browser);
        info!("chromium launched");

        Ok(())
    }

    async fn run_message_loop(
        self: Arc<Self>,
        mut receiver: mpsc::Receiver<Request>,
        app_state: Arc<AppState>,
    ) {
        while let Some(Request { message, reply }) = receiver.recv().await {
            info!("chrome message: {message:?}");

            let response = match self.handle_message(message, &app_state).await {
                Ok(response) => response,
                Err(error) => ChromeResponse::Error {
                    message: error.to_string(),
                },
            };

            let _ = reply.send(response);
        }

        error!("chrome message loop exited");
    }

    async fn handle_message(
        &self,
        message: ChromeMessage,
        app_state: &Arc<AppState>,
    ) -> Result<ChromeResponse> {
        match message {
            ChromeMessage::ActivatePlaylist { playlist_id } => {
                self.activate_playlist(&playlist_id, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::ActivateTab {
                tab_id,
                playlist_id,
            } => {
                self.activate_tab(&tab_id, &playlist_id, app_state).await?;
                self.hold(&playlist_id, app_state).await;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::NextTab => {
                self.step(1, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::PreviousTab => {
                self.step(-1, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::Pause => {
                self.stop_auto_rotation().await;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::Resume => {
                let playlist_id = self.state.lock().await.current_playlist_id.clone();

                if let Some(playlist_id) = playlist_id {
                    if let Some(playlist) = app_state.config.playlist(&playlist_id).await {
                        self.state.lock().await.hold_until = None;
                        self.start_auto_rotation(playlist.interval.into()).await;
                    }
                }

                Ok(ChromeResponse::Success)
            }
            ChromeMessage::RefreshTab { tab_id } => {
                self.refresh_tab(&tab_id).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::RecreateTab { tab_id } => {
                self.recreate_tab(&tab_id, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::CloseTab { tab_id } => {
                self.close_tab(&tab_id).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::GetStatus => {
                let state = self.state.lock().await;

                Ok(ChromeResponse::Status {
                    current_playlist_id: state.current_playlist_id.clone(),
                    current_tab_id: state.current_tab_id.clone(),
                    is_running: state.is_running,
                    auto_rotate: state.auto_rotate,
                })
            }
            ChromeMessage::Shutdown => {
                self.shutdown().await?;
                Ok(ChromeResponse::Success)
            }
        }
    }

    async fn activate_default_playlist(&self, app_state: &Arc<AppState>) -> Result<()> {
        let playlists = app_state.config.playlists().await;

        let Some(playlist) = playlists
            .iter()
            .find(|playlist| playlist.is_default)
            .or_else(|| playlists.first())
        else {
            warn!("no playlists configured");
            return Ok(());
        };

        self.activate_playlist(&playlist.playlist_id.clone(), app_state)
            .await
    }

    async fn activate_playlist(&self, playlist_id: &str, app_state: &Arc<AppState>) -> Result<()> {
        let playlist = app_state
            .config
            .playlist(playlist_id)
            .await
            .ok_or_else(|| anyhow!("playlist {playlist_id} not found"))?;

        let tabs = app_state.config.playlist_tabs(playlist_id).await;

        if tabs.is_empty() {
            return Err(anyhow!("playlist {playlist_id} has no enabled tabs"));
        }

        {
            let mut state = self.state.lock().await;

            state.current_playlist_id = Some(playlist_id.to_string());
            state.is_running = true;
            state.hold_until = None;
        }

        app_state
            .hass
            .publish_playlist_options(
                app_state
                    .config
                    .playlists()
                    .await
                    .iter()
                    .map(|playlist| playlist.playlist_id.clone())
                    .collect(),
                Some(playlist_id),
            )
            .await;

        for tab in tabs.iter().filter(|tab| tab.persist) {
            if !self.pages.lock().await.contains_key(&tab.tab_id) {
                if let Err(error) = self.create_tab_page(tab).await {
                    warn!("failed to preload tab {}: {error}", tab.tab_id);
                }
            }
        }

        self.activate_tab(&tabs[0].tab_id.clone(), playlist_id, app_state)
            .await?;
        self.start_auto_rotation(playlist.interval.into()).await;

        Ok(())
    }

    async fn activate_tab(
        &self,
        tab_id: &str,
        playlist_id: &str,
        app_state: &Arc<AppState>,
    ) -> Result<()> {
        let tab = app_state
            .config
            .tab(tab_id)
            .await
            .ok_or_else(|| anyhow!("tab {tab_id} not found"))?;

        if !self.pages.lock().await.contains_key(tab_id) {
            self.create_tab_page(&tab).await?;
        }

        let page = self
            .pages
            .lock()
            .await
            .get(tab_id)
            .cloned()
            .ok_or_else(|| anyhow!("no page for tab {tab_id}"))?;

        page.bring_to_front().await?;

        let previous = {
            let mut state = self.state.lock().await;
            let previous = state.current_tab_id.replace(tab_id.to_string());

            state.current_playlist_id = Some(playlist_id.to_string());
            state.current_tab_opened_at = Some(std::time::SystemTime::now());

            previous
        };

        if let Some(previous) = previous.filter(|previous| previous != tab_id) {
            self.set_capture_rate(&previous, None).await;
        }

        self.set_capture_rate(tab_id, Some(capture::FOREGROUND_FPS)).await;

        let tabs = app_state.config.playlist_tabs(playlist_id).await;

        app_state.hass.publish_tab_options(&tabs, Some(tab_id)).await;
        app_state.hass.publish_url(&tab.url).await;
        app_state.events.publish(app_state).await;

        Ok(())
    }

    async fn hold(&self, playlist_id: &str, app_state: &Arc<AppState>) {
        let Some(hold) = app_state
            .config
            .playlist(playlist_id)
            .await
            .and_then(|playlist| playlist.hold)
        else {
            return;
        };

        self.state.lock().await.hold_until = Some(Instant::now() + hold.into());
        info!("holding {playlist_id} for {hold}");
    }

    async fn create_tab_page(&self, tab: &Tab) -> Result<()> {
        let page = {
            let browser = self.browser.lock().await;
            let browser = browser.as_ref().ok_or_else(|| anyhow!("browser not ready"))?;

            browser.new_page(tab.url.as_str()).await?
        };

        if let Some(scale) = tab.scale {
            let (width, height) = self.output_size(&page).await.unwrap_or((1920, 1080));

            let _ = page
                .execute(
                    SetDeviceMetricsOverrideParams::builder()
                        .width(width as i64)
                        .height(height as i64)
                        .device_scale_factor(scale)
                        .mobile(false)
                        .build()
                        .map_err(|error| anyhow!("{error}"))?,
                )
                .await;
        }

        if let Some((width, height)) = self.output_size(&page).await {
            self.viewport
                .lock()
                .await
                .insert(tab.tab_id.clone(), (width, height));
        }

        self.pages
            .lock()
            .await
            .insert(tab.tab_id.clone(), page.clone());
        self.previews
            .lock()
            .await
            .insert(tab.tab_id.clone(), Preview::default());

        info!("opened page for tab {}", tab.tab_id);

        Ok(())
    }

    async fn output_size(&self, page: &Page) -> Option<(i32, i32)> {
        let width = page.evaluate("window.innerWidth").await.ok()?;
        let height = page.evaluate("window.innerHeight").await.ok()?;

        Some((
            width.value()?.as_u64()? as i32,
            height.value()?.as_u64()? as i32,
        ))
    }

    pub async fn watch_preview(&self, tab_id: &str) -> Option<watch::Receiver<Option<Vec<u8>>>> {
        let receiver = self
            .previews
            .lock()
            .await
            .get(tab_id)
            .map(|preview| preview.frames.subscribe())?;

        self.set_capture_rate(tab_id, Some(capture::MAX_FPS)).await;

        Some(receiver)
    }

    async fn set_capture_rate(&self, tab_id: &str, requested: Option<u64>) {
        let is_foreground = self.state.lock().await.current_tab_id.as_deref() == Some(tab_id);
        let mut previews = self.previews.lock().await;

        let Some(preview) = previews.get_mut(tab_id) else {
            return;
        };

        let fps = match (requested, is_foreground) {
            (Some(fps), _) => Some(fps),
            (None, true) => Some(capture::FOREGROUND_FPS),
            (None, false) if preview.has_viewers() => Some(capture::MAX_FPS),
            (None, false) => None,
        };

        let Some(fps) = fps else {
            self.stop_capture(preview, tab_id);

            return;
        };

        if preview.task.as_ref().is_some_and(|task| !task.is_finished()) {
            return;
        }

        let Some(page) = self.pages.lock().await.get(tab_id).cloned() else {
            return;
        };

        preview.task = Some(tokio::spawn(capture::run(
            page,
            preview.frames.clone(),
            fps,
            tab_id.to_string(),
            self.state.clone(),
        )));
    }

    fn stop_capture(&self, preview: &mut Preview, tab_id: &str) {
        if let Some(task) = preview.task.take() {
            task.abort();
        }

        let pages = self.pages.clone();
        let tab_id = tab_id.to_string();

        tokio::spawn(async move {
            if let Some(page) = pages.lock().await.get(&tab_id) {
                let _ = page.execute(StopScreencastParams::default()).await;
            }
        });
    }

    async fn step(&self, offset: i64, app_state: &Arc<AppState>) -> Result<()> {
        let (playlist_id, current_tab_id) = {
            let state = self.state.lock().await;

            (
                state.current_playlist_id.clone(),
                state.current_tab_id.clone(),
            )
        };

        let Some(playlist_id) = playlist_id else {
            return Ok(());
        };

        let tabs = app_state.config.playlist_tabs(&playlist_id).await;

        if tabs.is_empty() {
            return Ok(());
        }

        let current = current_tab_id
            .and_then(|tab_id| tabs.iter().position(|tab| tab.tab_id == tab_id))
            .unwrap_or(0) as i64;

        let count = tabs.len() as i64;
        let next = ((current + offset) % count + count) % count;

        self.activate_tab(&tabs[next as usize].tab_id.clone(), &playlist_id, app_state)
            .await
    }

    async fn start_auto_rotation(&self, interval: Duration) {
        if interval.is_zero() {
            return;
        }

        self.stop_auto_rotation().await;
        self.state.lock().await.auto_rotate = true;

        let sender = self.sender.clone();
        let state = self.state.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(align_to_wall_clock(interval)).await;

                {
                    let mut state = state.lock().await;

                    if !state.auto_rotate {
                        break;
                    }

                    match state.hold_until {
                        Some(until) if until > Instant::now() => continue,
                        Some(_) => state.hold_until = None,
                        None => {}
                    }
                }

                let (reply, _answer) = tokio::sync::oneshot::channel();

                if sender
                    .send(Request {
                        message: ChromeMessage::NextTab,
                        reply,
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        *self.auto_task.lock().await = Some(handle);
    }

    async fn stop_auto_rotation(&self) {
        self.state.lock().await.auto_rotate = false;

        if let Some(handle) = self.auto_task.lock().await.take() {
            handle.abort();
        }
    }

    async fn refresh_tab(&self, tab_id: &str) -> Result<()> {
        if let Some(page) = self.pages.lock().await.get(tab_id) {
            page.reload().await?;
        }

        Ok(())
    }

    async fn recreate_tab(&self, tab_id: &str, app_state: &Arc<AppState>) -> Result<()> {
        let tab = app_state
            .config
            .tab(tab_id)
            .await
            .ok_or_else(|| anyhow!("tab {tab_id} not found"))?;

        self.close_tab(tab_id).await?;
        self.create_tab_page(&tab).await
    }

    async fn close_tab(&self, tab_id: &str) -> Result<()> {
        if let Some(preview) = self.previews.lock().await.remove(tab_id) {
            if let Some(task) = preview.task {
                task.abort();
            }
        }

        // Dropping the handle leaves the Chromium target open. Closing it is what frees the
        // renderer process.
        if let Some(page) = self.pages.lock().await.remove(tab_id) {
            page.close().await?;
        }

        self.viewport.lock().await.remove(tab_id);

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        self.stop_auto_rotation().await;

        let tab_ids: Vec<String> = self.pages.lock().await.keys().cloned().collect();

        for tab_id in tab_ids {
            if let Err(error) = self.close_tab(&tab_id).await {
                warn!("failed to close tab {tab_id}: {error}");
            }
        }

        if let Some(mut browser) = self.browser.lock().await.take() {
            browser.close().await?;
            let _ = browser.wait().await;
        }

        Ok(())
    }

    pub async fn update_url(&self, tab_id: &str, url: &str) -> Result<()> {
        if let Some(page) = self.pages.lock().await.get(tab_id) {
            page.execute(
                NavigateParams::builder()
                    .url(url)
                    .build()
                    .map_err(|error| anyhow!("{error}"))?,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn viewport(&self, tab_id: &str) -> Option<(i32, i32)> {
        self.viewport.lock().await.get(tab_id).copied()
    }
}

fn align_to_wall_clock(interval: Duration) -> Duration {
    let seconds = interval.as_secs();

    if seconds == 0 {
        return interval;
    }

    let now = chrono::Local::now().timestamp() as u64;
    let remainder = now % seconds;

    if remainder == 0 {
        interval
    } else {
        Duration::from_secs(seconds - remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_clock_alignment_never_exceeds_the_interval() {
        for seconds in [1_u64, 30, 60, 300, 3600] {
            let interval = Duration::from_secs(seconds);
            let aligned = align_to_wall_clock(interval);

            assert!(aligned <= interval);
            assert!(!aligned.is_zero());
        }
    }

    #[test]
    fn a_zero_interval_is_left_alone() {
        assert!(align_to_wall_clock(Duration::ZERO).is_zero());
    }
}
