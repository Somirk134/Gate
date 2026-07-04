use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
    pub jwt_secret: String,
    pub tcp_port_range: PortRange,
    pub tls: TlsConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 5800,
            database_url: "sqlite://data/gate.db?mode=rwc".into(),
            log_level: "info".into(),
            jwt_secret: "change-me-in-production".into(),
            tcp_port_range: PortRange {
                start: 10000,
                end: 11000,
            },
            tls: TlsConfig {
                enabled: false,
                cert_path: "".into(),
                key_path: "".into(),
            },
            rate_limit: RateLimitConfig {
                requests_per_second: 100,
                burst_size: 200,
            },
        }
    }
}

pub fn load() -> Result<AppConfig> {
    let config_path = std::env::var("GATE_CONFIG")
        .unwrap_or_else(|_| "gate.toml".to_string());

    if Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        Ok(AppConfig::default())
    }
}
