use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;

fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("failed to spawn {} {:?}", cmd, args))?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "command {} {:?} exited with {}",
            cmd,
            args,
            status
        ));
    }
    Ok(())
}

pub fn set_dpms(on: bool, output: Option<&str>) -> Result<()> {
    let target = output.unwrap_or("*");
    let dpms_arg = if on { "on" } else { "off" };

    // Prefer swaymsg if available
    let swaymsg_result = Command::new("swaymsg")
        .args(["output", target, "dpms", dpms_arg])
        .status();

    if let Ok(status) = swaymsg_result {
        if status.success() {
            return Ok(());
        }
    }

    // Fallback to wlr-randr if swaymsg failed or missing
    run_command(
        "wlr-randr",
        &["--output", target, &format!("--{}", dpms_arg)],
    )
}

pub fn set_ddc_brightness(display: Option<&str>, value: f32) -> Result<()> {
    // DDC brightness VCP code 0x10 expects 0-100 integer
    let clamped = value.clamp(0.0, 1.0);
    let percent = (clamped * 100.0).round() as i32;
    let display_target = display.unwrap_or("1");

    // ddcutil setvcp 10 <percent> --display <display_target>
    let percent_str = percent.to_string();
    let mut args = vec!["setvcp", "10", &percent_str];
    args.push("--display");
    args.push(display_target);

    // ddcutil can be slow; add a small timeout via std::process::Command::status (inherits env timeout if set)
    run_command("ddcutil", &args)
}

pub fn delay_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
