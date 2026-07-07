use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};

pub fn get_app_data_dir() -> Result<PathBuf> {
    let base = platform_data_dir().context("unable to resolve app data directory")?;
    let path = base.join("Gate");
    fs::create_dir_all(&path).with_context(|| format!("unable to create {}", path.display()))?;
    Ok(path)
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .or_else(|| env::var_os("LOCALAPPDATA"))
        .map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn platform_data_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_data_dir() -> Option<PathBuf> {
    env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| {
        env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
    })
}
