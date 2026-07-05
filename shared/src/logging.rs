use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => Self::TRACE,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogOutput {
    Console,
    File { path: PathBuf },
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub outputs: Vec<LogOutput>,
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            outputs: vec![LogOutput::Console],
            format: LogFormat::Text,
        }
    }
}

pub trait Logger: Send + Sync {
    fn init(&self, config: &LoggingConfig) -> Result<(), ConfigError>;
}
