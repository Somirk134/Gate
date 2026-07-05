use crate::error::ConfigError;
use crate::health::HealthConfig;
use crate::logging::LoggingConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigSource {
    File(PathBuf),
    Environment { prefix: String },
    Cli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigPriority {
    Default,
    File,
    Environment,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliConfig {
    pub config_path: Option<PathBuf>,
    pub environment_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub runtime: RuntimeConfig,
    pub logging: LoggingConfig,
    pub health: HealthConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub instance_name: String,
    pub graceful_shutdown_timeout_seconds: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            instance_name: "gate-server".to_string(),
            graceful_shutdown_timeout_seconds: 30,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfig::default(),
            logging: LoggingConfig::default(),
            health: HealthConfig::default(),
        }
    }
}

pub trait ConfigCenter: Send + Sync {
    fn load(&self, sources: &[ConfigSource]) -> Result<AppConfig, ConfigError>;

    fn priorities(&self) -> &[ConfigPriority];
}
