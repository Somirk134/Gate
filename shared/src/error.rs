use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    Unknown,
    Config,
    Network,
    Tunnel,
    Internal,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::Unknown => "UNKNOWN",
            Self::Config => "CONFIG",
            Self::Network => "NETWORK",
            Self::Tunnel => "TUNNEL",
            Self::Internal => "INTERNAL",
        };

        f.write_str(code)
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    Tunnel(#[from] TunnelError),

    #[error(transparent)]
    Internal(#[from] InternalError),
}

impl AppError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Config(_) => ErrorCode::Config,
            Self::Network(_) => ErrorCode::Network,
            Self::Tunnel(_) => ErrorCode::Tunnel,
            Self::Internal(_) => ErrorCode::Internal,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration source is unavailable: {source}")]
    SourceUnavailable { source: String },

    #[error("configuration value is invalid: {key}")]
    InvalidValue { key: String },
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("network component is not configured")]
    NotConfigured,

    #[error("network component failed: {message}")]
    ComponentFailure { message: String },
}

#[derive(Debug, Error)]
pub enum TunnelError {
    #[error("tunnel capability is reserved for a future phase")]
    Reserved,
}

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("component is unavailable: {component}")]
    ComponentUnavailable { component: String },

    #[error("runtime invariant failed: {message}")]
    InvariantViolation { message: String },
}
