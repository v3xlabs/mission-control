use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt, FutureExt, pin_mut,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{error, info, warn};

use anyhow::Result;
use async_std::{
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
    db::repositories::{PlaylistRepository, TabRepository},
    state::AppState,
};

use super::{ChromeMessage, ChromeResponse, ChromeState};

pub struct ChromeController {
    pub state: Arc<Mutex<ChromeState>>,
    pub should_screen_capture: Arc<Mutex<bool>>,
    pub last_frame: Arc<Mutex<HashMap<String, Vec<u8>>>>, // Key is tab_id
    pub pages: Arc<Mutex<HashMap<String, Page>>>,        // Key is tab_id
    pub browser: Arc<Mutex<Option<Arc<Browser>>>>,
    pub message_sender: Sender<ChromeMessage>,
    pub message_receiver: Arc<Mutex<Receiver<ChromeMessage>>>,
    pub response_sender: Arc<Mutex<Option<Sender<ChromeResponse>>>>,
    pub timer_cancel: Arc<Mutex<Option<Sender<()>>>>,
}

impl ChromeController {
    pub fn new() -> Self {
        let (message_sender, message_receiver) = channel(100);

        Self {
            state: Arc::new(Mutex::new(ChromeState::default())),
            should_screen_capture: Arc::new(Mutex::new(true)),
            last_frame: Arc::new(Mutex::new(HashMap::new())),
            pages: Arc::new(Mutex::new(HashMap::new())),
            browser: Arc::new(Mutex::new(None)),
            message_sender,
            message_receiver: Arc::new(Mutex::new(message_receiver)),
            response_sender: Arc::new(Mutex::new(None)),
            timer_cancel: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_message_sender(&self) -> Sender<ChromeMessage> {
        self.message_sender.clone()
    }

    pub async fn start(self: &Arc<Self>, config: &ChromiumConfig, app_state: &Arc<AppState>) -> Result<()> {
        // Initialize Chrome browser
        self.initialize_browser(config).await?;

        // Start message handler - pass Arc<Self> to avoid clone issues
        let controller = self.clone();
        let app_state_clone = app_state.clone();
        let config_clone = config.clone();
        
        task::spawn(async move {
            controller.run_message_handler(app_state_clone, config_clone).await;
        });

        // Give the message handler time to start
        sleep(Duration::from_millis(100)).await;

        // Load initial tabs from database
        self.load_initial_tabs(app_state).await?;

        // Start with the first active playlist if available
        if let Ok(playlists) = app_state.playlist_repository.get_all().await {
            if let Some(active_playlist) = playlists.iter().find(|p| p.is_active) {
                info!("Found active playlist: {}, activating...", active_playlist.name);
                let _ = self.message_sender.clone().send(ChromeMessage::ActivatePlaylist {
                    playlist_id: active_playlist.id.clone(),
                }).await;
            } else {
                info!("No active playlist found");
            }
        }

        Ok(())
    }

    async fn initialize_browser(&self, config: &ChromiumConfig) -> Result<()> {
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
            browser_config = browser_config.arg("--disable-features=WebContentsForceDark");
        } else if theme == "dark" {
            browser_config = browser_config.arg("--enable-features=WebContentsForceDark:inversion_method/cielab_based/image_behavior/selective/text_lightness_threshold/150/background_lightness_threshold/205");
        }

        let browser_config = browser_config.build().map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?;
        let (browser_raw, mut handler) = Browser::launch(browser_config).await?;
        let browser = Arc::new(browser_raw);

        // Store browser for later use
        {
            let mut br_lock = self.browser.lock().await;
            *br_lock = Some(browser.clone());
        }

        // Handle browser events
        task::spawn(async move {
            while let Some(event) = handler.next().await {
                info!("Browser event: {:?}", event);
            }
        });

        // Give browser time to start
        sleep(Duration::from_secs(2)).await;

        info!("Chrome browser initialized successfully");
        Ok(())
    }

    async fn load_initial_tabs(&self, app_state: &Arc<AppState>) -> Result<()> {
        let tabs = app_state.tab_repository.get_all().await?;
        
        for tab in tabs {
            if tab.persist {
                self.create_tab_page(&tab.id, &tab.url).await?;
            }
        }

        Ok(())
    }

    async fn create_tab_page(&self, tab_id: &str, url: &str) -> Result<()> {
        let browser_opt = self.browser.lock().await;
        if let Some(browser) = browser_opt.as_ref() {
            info!("Creating new page for tab {} with URL: {}", tab_id, url);
            let page = browser.new_page("about:blank").await?;
            
            info!("Navigating to URL: {}", url);
            page.execute(
                NavigateParams::builder()
                    .url(url.to_string())
                    .build()
                    .unwrap(),
            )
            .await?;

            // Start screencasting if enabled
            let should_capture = *self.should_screen_capture.lock().await;
            info!("Screen capture enabled: {}", should_capture);
            
            if should_capture {
                info!("Starting screencast for tab: {}", tab_id);
                page.execute(
                    StartScreencastParams::builder()
                        .format(StartScreencastFormat::Jpeg)
                        .quality(80)
                        .build(),
                )
                .await?;

                let page_ref = page.clone();
                let last_frame = self.last_frame.clone();
                
                let tab_id_clone = tab_id.to_string();
                task::spawn(async move {
                    info!("Starting screencast event listener for tab: {}", tab_id_clone);
                    if let Ok(mut events) = page_ref.event_listener::<EventScreencastFrame>().await {
                        while let Some(frame) = events.next().await {
                            let frame_buf: &[u8] = frame.data.as_ref();
                            last_frame.lock().await.insert(tab_id_clone.clone(), frame_buf.to_vec());
                            info!("Captured frame for tab: {} (size: {} bytes)", tab_id_clone, frame_buf.len());

                            // Acknowledge the frame
                            let _ = page_ref.execute(
                                ScreencastFrameAckParams::builder()
                                    .session_id(frame.session_id)
                                    .build()
                                    .unwrap(),
                            ).await;
                        }
                    } else {
                        error!("Failed to create event listener for tab: {}", tab_id_clone);
                    }
                });
            }

            self.pages.lock().await.insert(tab_id.to_string(), page);
            info!("Created page for tab {} with URL: {}", tab_id, url);
        } else {
            error!("No browser available to create page for tab: {}", tab_id);
            return Err(anyhow::anyhow!("No browser available"));
        }

        Ok(())
    }

    async fn run_message_handler(&self, app_state: Arc<AppState>, config: ChromiumConfig) {
        let mut message_receiver = self.message_receiver.lock().await;
        info!("Chrome message handler started");
        
        while let Some(message) = message_receiver.next().await {
            info!("Chrome message handler received message: {:?}", message);
            match self.handle_message(message, &app_state, &config).await {
                Ok(response) => {
                    info!("Chrome message handled successfully, response: {:?}", response);
                    if let Some(sender) = &*self.response_sender.lock().await {
                        let _ = sender.clone().send(response).await;
                    }
                }
                Err(e) => {
                    error!("Error handling Chrome message: {}", e);
                    if let Some(sender) = &*self.response_sender.lock().await {
                        let _ = sender.clone().send(ChromeResponse::Error { 
                            message: e.to_string() 
                        }).await;
                    }
                }
            }
        }
    }



    async fn handle_message(&self, message: ChromeMessage, app_state: &Arc<AppState>, _config: &ChromiumConfig) -> Result<ChromeResponse> {
        info!("Processing Chrome message: {:?}", message);
        match message {
            ChromeMessage::ActivatePlaylist { playlist_id } => {
                self.activate_playlist(playlist_id, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::ActivateTab { tab_id, playlist_id } => {
                self.activate_tab(tab_id, playlist_id, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::StopPlaylist => {
                self.stop_playlist().await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::StartPlaylist => {
                self.start_playlist(app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::UpdateInterval { playlist_id, interval_seconds } => {
                self.update_interval(playlist_id, interval_seconds, app_state).await?;
                Ok(ChromeResponse::Success)
            }
            ChromeMessage::ReloadTab => {
                self.reload_current_tab().await?;
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

    async fn activate_playlist(&self, playlist_id: String, app_state: &Arc<AppState>) -> Result<()> {
        info!("Activating playlist: {}", playlist_id);
        
        // Get playlist and its tabs
        let playlist = match app_state.playlist_repository.get_by_id(&playlist_id).await? {
            Some(p) => p,
            None => return Err(anyhow::anyhow!("Playlist {} not found", playlist_id)),
        };
        
        let tabs = app_state.playlist_repository.get_tabs(&playlist_id).await?;
        info!("Found {} tabs in playlist {}", tabs.len(), playlist_id);

        if tabs.is_empty() {
            warn!("Playlist {} has no tabs", playlist_id);
            return Ok(());
        }

        // First deactivate all other playlists
        let all_playlists = app_state.playlist_repository.get_all().await?;
        for p in all_playlists {
            if p.id != playlist_id {
                app_state.playlist_repository.set_active(&p.id, false).await?;
            }
        }

        // Update playlist active status in database
        app_state.playlist_repository.set_active(&playlist_id, true).await?;
        info!("Updated database - playlist {} is now active", playlist_id);

        // Update state
        {
            let mut state = self.state.lock().await;
            state.current_playlist_id = Some(playlist_id.clone());
            state.current_tab_index = 0;
            state.is_running = true;
            info!("Updated controller state - playlist: {}, index: {}, running: {}", 
                playlist_id, state.current_tab_index, state.is_running);
        }

        // Start with first tab
        let first_tab = &tabs[0];
        info!("Activating first tab: {} ({}) in playlist {}", first_tab.name, first_tab.id, playlist_id);
        self.activate_tab(first_tab.id.clone(), playlist_id.clone(), app_state).await?;

        // Start auto-rotation if enabled
        if playlist.interval_seconds > 0 {
            info!("Starting auto-rotation for playlist {} with interval {} seconds", playlist_id, playlist.interval_seconds);
            self.start_auto_rotation(playlist_id, playlist.interval_seconds, app_state).await?;
        }

        Ok(())
    }

    async fn activate_tab(&self, tab_id: String, playlist_id: String, app_state: &Arc<AppState>) -> Result<()> {
        info!("Activating tab: {}", tab_id);

        // Get tab from database
        let tab = match app_state.tab_repository.get_by_id(&tab_id).await? {
            Some(t) => t,
            None => return Err(anyhow::anyhow!("Tab {} not found", tab_id)),
        };

        info!("Found tab in database: {} - {}", tab.name, tab.url);

        // Create page if it doesn't exist
        if !self.pages.lock().await.contains_key(&tab_id) {
            info!("Creating new page for tab: {}", tab_id);
            self.create_tab_page(&tab_id, &tab.url).await?;
        } else {
            info!("Page already exists for tab: {}", tab_id);
        }

        // Activate the page
        if let Some(page) = self.pages.lock().await.get(&tab_id) {
            info!("Activating page for tab: {}", tab_id);
            page.activate().await?;
            info!("Page activated successfully for tab: {}", tab_id);
        } else {
            error!("No page found for tab: {}", tab_id);
            return Err(anyhow::anyhow!("No page found for tab: {}", tab_id));
        }

        // Update state
        {
            let mut state = self.state.lock().await;
            state.current_tab_id = Some(tab_id.clone());
            state.current_playlist_id = Some(playlist_id.clone());
            info!("Updated controller state - current_tab_id: {}", tab_id);
        }

        // Update HASS entities
        app_state.hass.tab_entity.update_state(&app_state.hass.mqtt_client, &tab.name);
        app_state.hass.url_entity.update_state(&app_state.hass.mqtt_client, &tab.url);

        info!("Tab {} activated successfully", tab_id);
        Ok(())
    }

    async fn start_auto_rotation(&self, playlist_id: String, interval_seconds: i64, app_state: &Arc<AppState>) -> Result<()> {
        // Cancel existing timer
        if let Some(cancel_sender) = &*self.timer_cancel.lock().await {
            let _ = cancel_sender.clone().send(()).await;
        }

        // Create new timer
        let (cancel_sender, mut cancel_receiver) = channel(1);
        *self.timer_cancel.lock().await = Some(cancel_sender);

        let message_sender = self.message_sender.clone();
        let state = self.state.clone();
        let playlist_id_clone = playlist_id.clone();
        
        task::spawn(async move {
            loop {
                let sleep_future = sleep(Duration::from_secs(interval_seconds as u64)).fuse();
                let cancel_future = cancel_receiver.next().fuse();
                
                pin_mut!(sleep_future, cancel_future);
                
                futures::select! {
                    _ = sleep_future => {
                        // Check if we should continue rotating
                        let current_state = state.lock().await;
                        if current_state.auto_rotate && current_state.current_playlist_id.as_ref() == Some(&playlist_id_clone) {
                            drop(current_state);
                            let _ = message_sender.clone().send(ChromeMessage::NextTab).await;
                        } else {
                            break;
                        }
                    }
                    _ = cancel_future => {
                        info!("Auto-rotation cancelled for playlist {}", playlist_id_clone);
                        break;
                    }
                }
            }
        });

        // Update state
        {
            let mut state = self.state.lock().await;
            state.auto_rotate = true;
        }

        info!("Auto-rotation started for playlist {} with {} second intervals", playlist_id, interval_seconds);
        Ok(())
    }

    async fn stop_playlist(&self) -> Result<()> {
        info!("Stopping playlist");
        
        // Cancel timer
        if let Some(cancel_sender) = &*self.timer_cancel.lock().await {
            let _ = cancel_sender.clone().send(()).await;
        }

        // Update state
        {
            let mut state = self.state.lock().await;
            state.auto_rotate = false;
            state.is_running = false;
        }

        Ok(())
    }

    async fn start_playlist(&self, app_state: &Arc<AppState>) -> Result<()> {
        let current_playlist_id = {
            let state = self.state.lock().await;
            state.current_playlist_id.clone()
        };

        if let Some(playlist_id) = current_playlist_id {
            let playlist = app_state.playlist_repository.get_by_id(&playlist_id).await?;
            if let Some(playlist) = playlist {
                self.start_auto_rotation(playlist_id, playlist.interval_seconds, app_state).await?;
            }
        }

        Ok(())
    }

    async fn update_interval(&self, playlist_id: String, interval_seconds: i64, app_state: &Arc<AppState>) -> Result<()> {
        // Update in database
        app_state.playlist_repository.update_interval(&playlist_id, interval_seconds).await?;

        // Restart auto-rotation if this is the current playlist
        let current_playlist_id = {
            let state = self.state.lock().await;
            state.current_playlist_id.clone()
        };

        if current_playlist_id.as_ref() == Some(&playlist_id) {
            self.start_auto_rotation(playlist_id, interval_seconds, app_state).await?;
        }

        Ok(())
    }

    async fn next_tab(&self, app_state: &Arc<AppState>) -> Result<()> {
        let (current_playlist_id, current_index) = {
            let state = self.state.lock().await;
            (state.current_playlist_id.clone(), state.current_tab_index)
        };

        if let Some(playlist_id) = current_playlist_id {
            let tabs = app_state.playlist_repository.get_tabs(&playlist_id).await?;
            
            if !tabs.is_empty() {
                let next_index = (current_index + 1) % tabs.len();
                let next_tab = &tabs[next_index];
                
                self.activate_tab(next_tab.id.clone(), playlist_id.clone(), app_state).await?;
                
                // Update state
                {
                    let mut state = self.state.lock().await;
                    state.current_tab_index = next_index;
                }
            }
        }

        Ok(())
    }

    async fn previous_tab(&self, app_state: &Arc<AppState>) -> Result<()> {
        let (current_playlist_id, current_index) = {
            let state = self.state.lock().await;
            (state.current_playlist_id.clone(), state.current_tab_index)
        };

        if let Some(playlist_id) = current_playlist_id {
            let tabs = app_state.playlist_repository.get_tabs(&playlist_id).await?;
            
            if !tabs.is_empty() {
                let prev_index = if current_index == 0 {
                    tabs.len() - 1
                } else {
                    current_index - 1
                };
                let prev_tab = &tabs[prev_index];
                
                self.activate_tab(prev_tab.id.clone(), playlist_id.clone(), app_state).await?;
                
                // Update state
                {
                    let mut state = self.state.lock().await;
                    state.current_tab_index = prev_index;
                }
            }
        }

        Ok(())
    }

    async fn reload_current_tab(&self) -> Result<()> {
        let current_tab_id = {
            let state = self.state.lock().await;
            state.current_tab_id.clone()
        };

        if let Some(tab_id) = current_tab_id {
            if let Some(page) = self.pages.lock().await.get(&tab_id) {
                page.reload().await?;
            }
        }

        Ok(())
    }

    async fn update_tab_url(&self, tab_id: String, url: String, app_state: &Arc<AppState>) -> Result<()> {
        // Update in database
        app_state.tab_repository.update_url(&tab_id, &url).await?;

        // Update the page if it exists
        if let Some(page) = self.pages.lock().await.get(&tab_id) {
            page.execute(
                NavigateParams::builder()
                    .url(url.clone())
                    .build()
                    .unwrap(),
            )
            .await?;
        }

        Ok(())
    }

    async fn close_tab(&self, tab_id: String) -> Result<()> {
        if let Some(page) = self.pages.lock().await.remove(&tab_id) {
            page.close().await?;
        }

        // Remove from frame cache
        self.last_frame.lock().await.remove(&tab_id);

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Chrome controller");
        
        // Stop playlist
        self.stop_playlist().await?;

        // Close all pages
        let pages = self.pages.lock().await;
        for page in pages.values() {
            let _ = page.clone().close().await;
        }

        // Browser will be closed when Arc reference count drops to zero

        Ok(())
    }

    pub async fn get_screenshot(&self, tab_id: &str) -> Option<Vec<u8>> {
        self.last_frame.lock().await.get(tab_id).cloned()
    }
}

// Implement Clone for ChromeController
impl Clone for ChromeController {
    fn clone(&self) -> Self {
        let (message_sender, message_receiver) = channel(100);
        
        Self {
            state: self.state.clone(),
            should_screen_capture: self.should_screen_capture.clone(),
            last_frame: self.last_frame.clone(),
            pages: self.pages.clone(),
            browser: self.browser.clone(),
            message_sender,
            message_receiver: Arc::new(Mutex::new(message_receiver)),
            response_sender: Arc::new(Mutex::new(None)),
            timer_cancel: Arc::new(Mutex::new(None)),
        }
    }
} 