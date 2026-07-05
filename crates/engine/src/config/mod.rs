//! Configuration models for tunnels, protocols, forwarding, runtime, and heartbeat.

use crate::core::TunnelId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Supported protocol families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolKind {
    Http,
    Tcp,
    Https,
    Udp,
    P2p,
}

impl Default for ProtocolKind {
    fn default() -> Self {
        Self::Tcp
    }
}

/// Tunnel-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub id: TunnelId,
    pub name: String,
    pub protocol: ProtocolConfig,
    pub forward: ForwardConfig,
    pub heartbeat: HeartbeatConfig,
}

impl TunnelConfig {
    pub fn builder() -> TunnelConfigBuilder {
        TunnelConfigBuilder::default()
    }
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            id: TunnelId::new(),
            name: "tunnel".to_string(),
            protocol: ProtocolConfig::default(),
            forward: ForwardConfig::default(),
            heartbeat: HeartbeatConfig::default(),
        }
    }
}

/// Builder for [`TunnelConfig`].
#[derive(Debug, Clone)]
pub struct TunnelConfigBuilder {
    config: TunnelConfig,
}

impl Default for TunnelConfigBuilder {
    fn default() -> Self {
        Self {
            config: TunnelConfig::default(),
        }
    }
}

impl TunnelConfigBuilder {
    pub fn id(mut self, id: TunnelId) -> Self {
        self.config.id = id;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    pub fn protocol(mut self, protocol: ProtocolConfig) -> Self {
        self.config.protocol = protocol;
        self
    }

    pub fn forward(mut self, forward: ForwardConfig) -> Self {
        self.config.forward = forward;
        self
    }

    pub fn heartbeat(mut self, heartbeat: HeartbeatConfig) -> Self {
        self.config.heartbeat = heartbeat;
        self
    }

    pub fn build(self) -> TunnelConfig {
        self.config
    }
}

/// Protocol binding and target configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub kind: ProtocolKind,
    pub bind_host: String,
    pub bind_port: u16,
    pub target_host: String,
    pub target_port: u16,
}

impl ProtocolConfig {
    pub fn builder() -> ProtocolConfigBuilder {
        ProtocolConfigBuilder::default()
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            kind: ProtocolKind::Tcp,
            bind_host: "127.0.0.1".to_string(),
            bind_port: 0,
            target_host: "127.0.0.1".to_string(),
            target_port: 0,
        }
    }
}

/// Builder for [`ProtocolConfig`].
#[derive(Debug, Clone)]
pub struct ProtocolConfigBuilder {
    config: ProtocolConfig,
}

impl Default for ProtocolConfigBuilder {
    fn default() -> Self {
        Self {
            config: ProtocolConfig::default(),
        }
    }
}

impl ProtocolConfigBuilder {
    pub fn kind(mut self, kind: ProtocolKind) -> Self {
        self.config.kind = kind;
        self
    }

    pub fn bind(mut self, host: impl Into<String>, port: u16) -> Self {
        self.config.bind_host = host.into();
        self.config.bind_port = port;
        self
    }

    pub fn target(mut self, host: impl Into<String>, port: u16) -> Self {
        self.config.target_host = host.into();
        self.config.target_port = port;
        self
    }

    pub fn build(self) -> ProtocolConfig {
        self.config
    }
}

/// Forwarding behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardConfig {
    pub buffer_size: usize,
    pub flush_interval: Duration,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

impl ForwardConfig {
    pub fn builder() -> ForwardConfigBuilder {
        ForwardConfigBuilder::default()
    }
}

impl Default for ForwardConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024,
            flush_interval: Duration::from_millis(100),
            compression_enabled: false,
            encryption_enabled: false,
        }
    }
}

/// Builder for [`ForwardConfig`].
#[derive(Debug, Clone)]
pub struct ForwardConfigBuilder {
    config: ForwardConfig,
}

impl Default for ForwardConfigBuilder {
    fn default() -> Self {
        Self {
            config: ForwardConfig::default(),
        }
    }
}

impl ForwardConfigBuilder {
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.config.buffer_size = buffer_size;
        self
    }

    pub fn flush_interval(mut self, flush_interval: Duration) -> Self {
        self.config.flush_interval = flush_interval;
        self
    }

    pub fn compression_enabled(mut self, enabled: bool) -> Self {
        self.config.compression_enabled = enabled;
        self
    }

    pub fn encryption_enabled(mut self, enabled: bool) -> Self {
        self.config.encryption_enabled = enabled;
        self
    }

    pub fn build(self) -> ForwardConfig {
        self.config
    }
}

/// Runtime execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub worker_threads: usize,
    pub max_tasks: usize,
    pub shutdown_timeout: Duration,
}

impl RuntimeConfig {
    pub fn builder() -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::default()
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: 4,
            max_tasks: 4096,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

/// Builder for [`RuntimeConfig`].
#[derive(Debug, Clone)]
pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self {
            config: RuntimeConfig::default(),
        }
    }
}

impl RuntimeConfigBuilder {
    pub fn worker_threads(mut self, worker_threads: usize) -> Self {
        self.config.worker_threads = worker_threads;
        self
    }

    pub fn max_tasks(mut self, max_tasks: usize) -> Self {
        self.config.max_tasks = max_tasks;
        self
    }

    pub fn shutdown_timeout(mut self, shutdown_timeout: Duration) -> Self {
        self.config.shutdown_timeout = shutdown_timeout;
        self
    }

    pub fn build(self) -> RuntimeConfig {
        self.config
    }
}

/// Heartbeat scheduling and timeout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub interval: Duration,
    pub timeout: Duration,
    pub max_missed_ticks: u32,
}

impl HeartbeatConfig {
    pub fn builder() -> HeartbeatConfigBuilder {
        HeartbeatConfigBuilder::default()
    }
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(30),
            max_missed_ticks: 3,
        }
    }
}

/// Builder for [`HeartbeatConfig`].
#[derive(Debug, Clone)]
pub struct HeartbeatConfigBuilder {
    config: HeartbeatConfig,
}

impl Default for HeartbeatConfigBuilder {
    fn default() -> Self {
        Self {
            config: HeartbeatConfig::default(),
        }
    }
}

impl HeartbeatConfigBuilder {
    pub fn interval(mut self, interval: Duration) -> Self {
        self.config.interval = interval;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn max_missed_ticks(mut self, max_missed_ticks: u32) -> Self {
        self.config.max_missed_ticks = max_missed_ticks;
        self
    }

    pub fn build(self) -> HeartbeatConfig {
        self.config
    }
}
