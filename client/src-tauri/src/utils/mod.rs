use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};

const DATA_DIR_ENV: &str = "GATE_DATA_DIR";
const RELEASE_DATA_DIR_NAME: &str = "Gate-release";
const DEV_DATA_DIR_NAME: &str = "Gate-dev";

pub fn get_app_data_dir() -> Result<PathBuf> {
    let path = app_data_dir().context("unable to resolve app data directory")?;
    fs::create_dir_all(&path).with_context(|| format!("unable to create {}", path.display()))?;
    Ok(path)
}

// 统一持久化目录入口：开发版和正式版必须隔离，避免打包后的程序读取开发期数据。
pub(crate) fn app_data_dir() -> Option<PathBuf> {
    if let Some(path) = explicit_data_dir() {
        return Some(path);
    }

    platform_data_dir().map(|base| base.join(app_data_dir_name()))
}

fn explicit_data_dir() -> Option<PathBuf> {
    env::var_os(DATA_DIR_ENV)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

fn app_data_dir_name() -> &'static str {
    if cfg!(debug_assertions) {
        DEV_DATA_DIR_NAME
    } else {
        RELEASE_DATA_DIR_NAME
    }
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
