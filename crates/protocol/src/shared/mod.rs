use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{error::ErrorCode, version::ProtocolVersion};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ClientInfo {
    pub client_id: Option<String>,
    pub version: Option<String>,
    pub platform: Option<String>,
    pub os: Option<String>,
    pub language: Option<String>,
    pub architecture: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServerInfo {
    pub server_id: Option<String>,
    pub version: Option<String>,
    pub region: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: ProtocolVersion,
    pub supported: Vec<ProtocolVersion>,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            current: ProtocolVersion::V1,
            supported: vec![ProtocolVersion::V1],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub name: Option<String>,
    pub os: Option<String>,
    pub architecture: Option<String>,
    pub runtime: Option<String>,
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: ErrorCode,
    pub code_value: u16,
    pub message: String,
    pub details: BTreeMap<String, String>,
}

impl ErrorInfo {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            code_value: code.as_u16(),
            message: message.into(),
            details: BTreeMap::new(),
        }
    }
}
