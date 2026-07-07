use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::utils::get_app_data_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_addr: String,
    pub auth_token: String,
    pub theme: String,
    pub language: String,
    pub auto_connect: bool,
    pub minimize_to_tray: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_addr: String::new(),
            auth_token: String::new(),
            theme: "light".into(),
            language: "en".into(),
            auto_connect: false,
            minimize_to_tray: true,
        }
    }
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let raw = serde_json::to_string_pretty(config)?;
    fs::write(path, raw)?;
    Ok(())
}

fn config_path() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("client-config.json"))
}
