//! Heartbeat manager boundary.

use crate::config::HeartbeatConfig;
use crate::core::TunnelId;
use crate::error::EngineError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartbeatStatus {
    Idle,
    Running,
    Stopped,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct HeartbeatManager {
    config: HeartbeatConfig,
}

impl Default for HeartbeatManager {
    fn default() -> Self {
        Self {
            config: HeartbeatConfig::default(),
        }
    }
}

impl HeartbeatManager {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self { config }
    }

    pub fn start(&self, _tunnel_id: TunnelId) -> Result<(), EngineError> {
        Ok(())
    }

    pub fn stop(&self, _tunnel_id: TunnelId) -> Result<(), EngineError> {
        Ok(())
    }

    pub fn tick(&self, _tunnel_id: TunnelId) -> Result<(), EngineError> {
        Ok(())
    }

    pub fn check_timeout(&self, _tunnel_id: TunnelId) -> Result<HeartbeatStatus, EngineError> {
        Ok(HeartbeatStatus::Idle)
    }

    pub fn config(&self) -> &HeartbeatConfig {
        &self.config
    }
}
