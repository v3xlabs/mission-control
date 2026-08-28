pub mod capture;
pub mod schedule;

use std::{
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use tokio::process::Command;
use tracing::{info, warn};

use crate::config::DisplayDocument;

/// `ddcutil` routinely takes over a second, and a compositor command can hang if the socket is
/// gone. Neither may stall the request that triggered it.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Display {
    on: AtomicBool,
    brightness: AtomicU32,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    pub fn new() -> Self {
        Self {
            on: AtomicBool::new(true),
            brightness: AtomicU32::new(100),
        }
    }

    pub fn is_on(&self) -> bool {
        self.on.load(Ordering::Relaxed)
    }

    pub fn brightness(&self) -> u32 {
        self.brightness.load(Ordering::Relaxed)
    }

    pub async fn set_power(&self, config: &DisplayDocument, on: bool) -> Result<()> {
        let template = if on {
            &config.power_on
        } else {
            &config.power_off
        };

        run(&substitute(template, config.output.as_deref(), None)).await?;
        self.on.store(on, Ordering::Relaxed);
        info!(on, "display power");

        Ok(())
    }

    pub async fn set_brightness(&self, config: &DisplayDocument, percent: u32) -> Result<()> {
        let percent = percent.min(100);

        run(&substitute(
            &config.brightness,
            config.output.as_deref(),
            Some(percent),
        ))
        .await?;
        self.brightness.store(percent, Ordering::Relaxed);
        info!(percent, "display brightness");

        Ok(())
    }
}

pub(super) fn substitute(
    template: &[String],
    output: Option<&str>,
    percent: Option<u32>,
) -> Vec<String> {
    template
        .iter()
        .map(|part| {
            let part = match percent {
                Some(percent) => part.replace("{percent}", &percent.to_string()),
                None => part.clone(),
            };

            match output {
                Some(output) => part.replace("{output}", output),
                None => part,
            }
        })
        .collect()
}

async fn run(command: &[String]) -> Result<()> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| anyhow!("empty command"))?;

    let status = tokio::time::timeout(COMMAND_TIMEOUT, Command::new(program).args(args).status())
        .await
        .map_err(|_| anyhow!("{program} did not finish within {COMMAND_TIMEOUT:?}"))?
        .with_context(|| format!("failed to spawn {program}"))?;

    if !status.success() {
        return Err(anyhow!("{program} exited with {status}"));
    }

    Ok(())
}

pub async fn logind(action: &str) -> Result<()> {
    warn!(action, "requesting power action over logind");

    run(&[
        "busctl".to_string(),
        "call".to_string(),
        "org.freedesktop.login1".to_string(),
        "/org/freedesktop/login1".to_string(),
        "org.freedesktop.login1.Manager".to_string(),
        action.to_string(),
        "b".to_string(),
        "false".to_string(),
    ])
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    #[test]
    fn percent_and_output_are_substituted() {
        let command = substitute(
            &words(&[
                "ddcutil",
                "setvcp",
                "10",
                "{percent}",
                "--display",
                "{output}",
            ]),
            Some("DP-1"),
            Some(65),
        );

        assert_eq!(
            command,
            words(&["ddcutil", "setvcp", "10", "65", "--display", "DP-1"])
        );
    }

    #[test]
    fn a_template_without_placeholders_is_unchanged() {
        let template = words(&["niri", "msg", "action", "power-off-monitors"]);

        assert_eq!(substitute(&template, Some("DP-1"), None), template);
    }

    #[tokio::test]
    async fn an_empty_command_is_an_error() {
        assert!(run(&[]).await.is_err());
    }

    #[tokio::test]
    async fn a_failing_command_is_an_error() {
        assert!(run(&words(&["false"])).await.is_err());
    }
}
