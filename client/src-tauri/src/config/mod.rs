use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    todo!("load config from app data directory")
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    todo!("save config to app data directory")
}
