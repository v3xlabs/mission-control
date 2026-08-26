use std::{sync::Arc, time::Duration};

use anyhow::Result;
use tokio::signal::unix::SignalKind;
use tracing::{error, info, warn};

use crate::{
    chrome::{tell, ChromeMessage},
    config::Dirs,
    display::schedule::{self, Baseline},
    state::AppState,
};

pub mod api;
pub mod chrome;
pub mod config;
pub mod db;
pub mod display;
pub mod events;
pub mod hass;
pub mod http;
pub mod notifications;
pub mod player;
pub mod state;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let dirs = Dirs::resolve()?;
    let state = AppState::new(dirs).await?;

    if state.admin_key.is_none() {
        warn!("no admin_key configured: every mutation on this port is unauthenticated");
    }

    if state.config.is_read_only() {
        info!("config directory is read-only: changes apply until restart only");
    }

    if state.config.read().await.device.chromium.enabled {
        let controller = state.chrome.clone();
        let started = state.clone();

        tokio::spawn(async move {
            if let Err(error) = controller.start(&started).await {
                error!("failed to start the chrome controller: {error}");
            }
        });
    }

    tokio::spawn(run_schedule(state.clone()));

    if state.hass.is_enabled() {
        let hass = state.hass.clone();
        let hass_state = state.clone();

        tokio::spawn(async move { hass.run(hass_state).await });
    } else {
        info!("home assistant integration disabled");
    }

    let server = state.clone();
    let http = tokio::spawn(async move {
        if let Err(error) = http::start_http(server).await {
            error!("http server stopped: {error}");
        }
    });

    tokio::select! {
        _ = http => {}
        signal = shutdown_signal() => info!(%signal, "shutting down"),
    }

    // Without this the browser survives the daemon and the next start finds a second one.
    if let Err(error) = tell(&state.chrome, ChromeMessage::Shutdown).await {
        warn!("failed to shut the browser down cleanly: {error}");
    }

    state.player.shutdown().await;
    state.overlay.shutdown().await;
    state.sidebar.hide().await;

    Ok(())
}

/// systemd stops a service with SIGTERM, so listening for SIGINT alone means every `systemctl
/// restart` kills the daemon before it can close the browser. Chromium is then killed with the
/// rest of the cgroup and leaves its profile locked against the next start.
async fn shutdown_signal() -> &'static str {
    let mut terminate = match tokio::signal::unix::signal(SignalKind::terminate()) {
        Ok(signal) => signal,
        Err(error) => {
            error!("cannot listen for SIGTERM: {error}");

            return "none";
        }
    };

    tokio::select! {
        _ = terminate.recv() => "SIGTERM",
        result = tokio::signal::ctrl_c() => {
            if let Err(error) = result {
                error!("cannot listen for SIGINT: {error}");
            }

            "SIGINT"
        }
    }
}

async fn run_schedule(state: Arc<AppState>) {
    let mut last_baseline = None;

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        let display = state.config.read().await.display;
        let baseline = schedule::baseline_at(&display.schedule, chrono::Local::now());

        if baseline == Baseline::Unscheduled {
            continue;
        }

        // Only act on a crossing, so a manual override survives until the next boundary.
        if last_baseline == Some(baseline) {
            continue;
        }

        last_baseline = Some(baseline);

        let on = baseline == Baseline::On;

        if let Err(error) = state.display.set_power(&display, on).await {
            warn!("schedule failed to set display power: {error}");
            continue;
        }

        state.hass.publish_backlight(on).await;
        state.events.publish(&state).await;
    }
}
