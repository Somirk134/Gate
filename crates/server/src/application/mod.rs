pub mod services;
pub mod use_cases;

use crate::config::AppConfig;
use anyhow::Result;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}
