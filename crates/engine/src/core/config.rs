//! Core engine configuration.

use crate::config::{HeartbeatConfig, RuntimeConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Engine-level configuration shared by all modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub name: String,
    pub runtime: RuntimeConfig,
    pub heartbeat: HeartbeatConfig,
    pub max_tunnels: usize,
    pub labels: BTreeMap<String, String>,
}

impl EngineConfig {
    pub fn builder() -> EngineConfigBuilder {
        EngineConfigBuilder::default()
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            name: "gate-engine".to_string(),
            runtime: RuntimeConfig::default(),
            heartbeat: HeartbeatConfig::default(),
            max_tunnels: 1024,
            labels: BTreeMap::new(),
        }
    }
}

/// Builder for [`EngineConfig`].
#[derive(Debug, Clone)]
pub struct EngineConfigBuilder {
    config: EngineConfig,
}

impl Default for EngineConfigBuilder {
    fn default() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }
}

impl EngineConfigBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    pub fn runtime(mut self, runtime: RuntimeConfig) -> Self {
        self.config.runtime = runtime;
        self
    }

    pub fn heartbeat(mut self, heartbeat: HeartbeatConfig) -> Self {
        self.config.heartbeat = heartbeat;
        self
    }

    pub fn max_tunnels(mut self, max_tunnels: usize) -> Self {
        self.config.max_tunnels = max_tunnels;
        self
    }

    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.labels.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> EngineConfig {
        self.config
    }
}
