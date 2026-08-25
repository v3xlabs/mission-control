use std::path::PathBuf;

use anyhow::{anyhow, Result};

pub struct Dirs {
    pub config: PathBuf,
    pub state: PathBuf,
    pub cache: PathBuf,
}

impl Dirs {
    pub fn resolve() -> Result<Self> {
        Ok(Self {
            config: resolve_one("MISSIOND_CONFIG_DIR", "XDG_CONFIG_HOME", ".config")?,
            state: resolve_one("MISSIOND_STATE_DIR", "XDG_STATE_HOME", ".local/state")?,
            cache: resolve_one("MISSIOND_CACHE_DIR", "XDG_CACHE_HOME", ".cache")?,
        })
    }

    pub fn create_state_and_cache(&self) -> Result<()> {
        std::fs::create_dir_all(&self.state)?;
        std::fs::create_dir_all(&self.cache)?;
        Ok(())
    }
}

fn resolve_one(own: &str, xdg: &str, home_relative: &str) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(own) {
        return Ok(PathBuf::from(path));
    }
    if let Some(base) = std::env::var_os(xdg) {
        return Ok(PathBuf::from(base).join("missiond"));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| anyhow!("cannot resolve {own}: neither {xdg} nor HOME is set"))?;
    Ok(PathBuf::from(home).join(home_relative).join("missiond"))
}
