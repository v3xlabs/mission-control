use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use base64::engine::{general_purpose::STANDARD as BASE64, Engine as _};
use chromiumoxide::{
    cdp::browser_protocol::page::{
        EventScreencastFrame, ScreencastFrameAckParams, StartScreencastFormat,
        StartScreencastParams, StopScreencastParams,
    },
    Page,
};
use futures::StreamExt;
use tokio::sync::{watch, Mutex};
use tracing::warn;

use super::ChromeState;

pub const MAX_FPS: u64 = 4;
pub const FOREGROUND_FPS: u64 = 1;

const MAX_ENCODED_BYTES: usize = 5_000_000;

pub async fn run(
    page: Page,
    frames: watch::Sender<Option<Vec<u8>>>,
    fps: u64,
    tab_id: String,
    state: Arc<Mutex<ChromeState>>,
) {
    let started = page
        .execute(
            StartScreencastParams::builder()
                .format(StartScreencastFormat::Jpeg)
                .quality(80)
                .build(),
        )
        .await;

    if let Err(error) = started {
        warn!("screencast start failed for {tab_id}: {error:?}");
        return;
    }

    let Ok(mut events) = page.event_listener::<EventScreencastFrame>().await else {
        warn!("screencast listener failed for {tab_id}");
        return;
    };

    let minimum_interval = Duration::from_millis(1000 / fps.max(1));
    let mut last = Instant::now() - minimum_interval;

    while let Some(frame) = events.next().await {
        // Chromium sends no further frames until the last one is acknowledged, so this happens
        // before any decision to skip it.
        if let Ok(ack) = ScreencastFrameAckParams::builder()
            .session_id(frame.session_id)
            .build()
        {
            let _ = page.execute(ack).await;
        }

        let encoded: &[u8] = frame.data.as_ref();
        let now = Instant::now();

        if encoded.len() > MAX_ENCODED_BYTES || now.duration_since(last) < minimum_interval {
            continue;
        }

        last = now;

        // CDP sends the frame base64 encoded and chromiumoxide hands it over untouched.
        match BASE64.decode(encoded) {
            Ok(jpeg) => {
                let _ = frames.send(Some(jpeg));
            }
            Err(error) => warn!("screencast frame for {tab_id} was not base64: {error}"),
        }

        if is_unwanted(&frames, &tab_id, &state).await {
            let _ = page.execute(StopScreencastParams::default()).await;

            break;
        }
    }
}

/// The sender keeps one receiver of its own, so anything above that count is a real viewer.
async fn is_unwanted(
    frames: &watch::Sender<Option<Vec<u8>>>,
    tab_id: &str,
    state: &Arc<Mutex<ChromeState>>,
) -> bool {
    frames.receiver_count() <= 1
        && state.lock().await.current_tab_id.as_deref() != Some(tab_id)
}
