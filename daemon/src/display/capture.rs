use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use tokio::{process::Command, sync::Mutex};
use tracing::warn;

use crate::config::DisplayDocument;

/// Grabbing the output spawns a process, so bursts of requests collapse onto one capture. A
/// display changes on the order of seconds; this only stops a browser holding the page open from
/// spawning `grim` on every animation frame.
const MIN_INTERVAL: Duration = Duration::from_millis(900);

/// A compositor that has gone away leaves the capture tool waiting on a socket that will never
/// answer.
const TIMEOUT: Duration = Duration::from_secs(10);

/// What the compositor is actually putting on the panel, as opposed to what one page painted.
///
/// This goes through the compositor's own screencopy protocol rather than through the browser, so
/// it captures the whole output including anything drawn over the page. It runs only when asked.
pub struct OutputCapture {
    last: Mutex<Option<(Instant, Vec<u8>)>>,
}

impl Default for OutputCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputCapture {
    pub fn new() -> Self {
        Self {
            last: Mutex::new(None),
        }
    }

    pub async fn grab(&self, config: &DisplayDocument) -> Result<Vec<u8>> {
        let mut last = self.last.lock().await;

        if let Some((taken_at, image)) = last.as_ref() {
            if taken_at.elapsed() < MIN_INTERVAL {
                return Ok(image.clone());
            }
        }

        let image = run(&super::substitute(
            &config.screenshot,
            config.output.as_deref(),
            None,
        ))
        .await?;

        *last = Some((Instant::now(), image.clone()));

        Ok(image)
    }
}

async fn run(command: &[String]) -> Result<Vec<u8>> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| anyhow!("empty screenshot command"))?;

    let output = tokio::time::timeout(TIMEOUT, Command::new(program).args(args).output())
        .await
        .map_err(|_| anyhow!("{program} did not finish within {TIMEOUT:?}"))?
        .with_context(|| format!("failed to spawn {program}"))?;

    if !output.status.success() {
        let reason = String::from_utf8_lossy(&output.stderr);

        warn!(%reason, "screenshot command failed");

        return Err(anyhow!("{program} exited with {}", output.status));
    }

    if output.stdout.is_empty() {
        return Err(anyhow!("{program} wrote no image"));
    }

    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_empty_command_is_an_error() {
        assert!(run(&[]).await.is_err());
    }

    #[tokio::test]
    async fn a_command_that_writes_nothing_is_an_error() {
        let command = ["true"].map(String::from).to_vec();

        assert!(run(&command).await.is_err());
    }

    #[tokio::test]
    async fn a_failing_command_is_an_error() {
        let command = ["false"].map(String::from).to_vec();

        assert!(run(&command).await.is_err());
    }

    #[tokio::test]
    async fn output_is_returned_verbatim() {
        let command = ["printf", "not-really-a-jpeg"].map(String::from).to_vec();

        assert_eq!(run(&command).await.unwrap(), b"not-really-a-jpeg");
    }

    #[tokio::test]
    async fn a_second_request_inside_the_interval_reuses_the_capture() {
        let capture = OutputCapture::new();
        let config = DisplayDocument {
            // `date` changes every second, so a fresh run would differ from a reused one.
            screenshot: vec!["date".to_string(), "+%s%N".to_string()],
            ..DisplayDocument::default()
        };

        let first = capture.grab(&config).await.unwrap();
        let second = capture.grab(&config).await.unwrap();

        assert_eq!(first, second);
    }
}
