//! Configuration models for tunnels, protocols, forwarding, runtime, heartbeat,
//! reconnect, health, and state synchronization.

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

/// Heartbeat scheduling, timeout, and retry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Interval between heartbeat ticks.
    pub interval: Duration,
    /// Maximum time to wait for a pong after a ping.
    pub timeout: Duration,
    /// Number of heartbeat retries before reconnect is expected to take over.
    pub retry_count: u32,
    /// Delay between heartbeat retries.
    pub retry_delay: Duration,
    /// Maximum missed heartbeat count before the connection is considered lost.
    pub max_missed_heartbeat: u32,
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
            timeout: Duration::from_secs(15),
            retry_count: 3,
            retry_delay: Duration::from_secs(2),
            max_missed_heartbeat: 3,
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

    pub fn retry_count(mut self, retry_count: u32) -> Self {
        self.config.retry_count = retry_count;
        self
    }

    pub fn retry_delay(mut self, retry_delay: Duration) -> Self {
        self.config.retry_delay = retry_delay;
        self
    }

    pub fn max_missed_heartbeat(mut self, max_missed_heartbeat: u32) -> Self {
        self.config.max_missed_heartbeat = max_missed_heartbeat;
        self
    }

    pub fn max_missed_ticks(mut self, max_missed_ticks: u32) -> Self {
        self.config.max_missed_heartbeat = max_missed_ticks;
        self
    }

    pub fn build(self) -> HeartbeatConfig {
        self.config
    }
}

/// Static reconnect strategy configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReconnectStrategyConfig {
    Immediate,
    Linear {
        delay: Duration,
    },
    ExponentialBackoff {
        base_delay: Duration,
        max_delay: Duration,
        factor: f64,
    },
    FixedInterval {
        interval: Duration,
    },
    Custom {
        delays: Vec<Duration>,
    },
}

impl Default for ReconnectStrategyConfig {
    fn default() -> Self {
        Self::ExponentialBackoff {
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            factor: 2.0,
        }
    }
}

/// Reconnect queue, scheduler, and session restoration configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    pub auto_reconnect: bool,
    pub max_attempts: u32,
    pub queue_capacity: usize,
    pub strategy: ReconnectStrategyConfig,
    pub scheduler_tick: Duration,
    pub recover_session: bool,
    pub recover_tunnel: bool,
    pub recover_statistics: bool,
    pub recover_context: bool,
    pub recover_subscription: bool,
}

impl ReconnectConfig {
    pub fn builder() -> ReconnectConfigBuilder {
        ReconnectConfigBuilder::default()
    }
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            max_attempts: 10,
            queue_capacity: 65_536,
            strategy: ReconnectStrategyConfig::default(),
            scheduler_tick: Duration::from_millis(100),
            recover_session: true,
            recover_tunnel: true,
            recover_statistics: true,
            recover_context: true,
            recover_subscription: true,
        }
    }
}

/// Builder for [`ReconnectConfig`].
#[derive(Debug, Clone)]
pub struct ReconnectConfigBuilder {
    config: ReconnectConfig,
}

impl Default for ReconnectConfigBuilder {
    fn default() -> Self {
        Self {
            config: ReconnectConfig::default(),
        }
    }
}

impl ReconnectConfigBuilder {
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.config.max_attempts = max_attempts;
        self
    }

    pub fn queue_capacity(mut self, queue_capacity: usize) -> Self {
        self.config.queue_capacity = queue_capacity;
        self
    }

    pub fn strategy(mut self, strategy: ReconnectStrategyConfig) -> Self {
        self.config.strategy = strategy;
        self
    }

    pub fn scheduler_tick(mut self, scheduler_tick: Duration) -> Self {
        self.config.scheduler_tick = scheduler_tick;
        self
    }

    pub fn recover_session(mut self, enabled: bool) -> Self {
        self.config.recover_session = enabled;
        self
    }

    pub fn recover_tunnel(mut self, enabled: bool) -> Self {
        self.config.recover_tunnel = enabled;
        self
    }

    pub fn recover_statistics(mut self, enabled: bool) -> Self {
        self.config.recover_statistics = enabled;
        self
    }

    pub fn recover_context(mut self, enabled: bool) -> Self {
        self.config.recover_context = enabled;
        self
    }

    pub fn recover_subscription(mut self, enabled: bool) -> Self {
        self.config.recover_subscription = enabled;
        self
    }

    pub fn build(self) -> ReconnectConfig {
        self.config
    }
}

/// Health aggregation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub enabled: bool,
    pub check_interval: Duration,
    pub warning_score_threshold: u8,
    pub critical_score_threshold: u8,
    pub offline_after: Duration,
}

impl HealthConfig {
    pub fn builder() -> HealthConfigBuilder {
        HealthConfigBuilder::default()
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(5),
            warning_score_threshold: 80,
            critical_score_threshold: 50,
            offline_after: Duration::from_secs(45),
        }
    }
}

/// Builder for [`HealthConfig`].
#[derive(Debug, Clone)]
pub struct HealthConfigBuilder {
    config: HealthConfig,
}

impl Default for HealthConfigBuilder {
    fn default() -> Self {
        Self {
            config: HealthConfig::default(),
        }
    }
}

impl HealthConfigBuilder {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    pub fn check_interval(mut self, check_interval: Duration) -> Self {
        self.config.check_interval = check_interval;
        self
    }

    pub fn warning_score_threshold(mut self, threshold: u8) -> Self {
        self.config.warning_score_threshold = threshold;
        self
    }

    pub fn critical_score_threshold(mut self, threshold: u8) -> Self {
        self.config.critical_score_threshold = threshold;
        self
    }

    pub fn offline_after(mut self, offline_after: Duration) -> Self {
        self.config.offline_after = offline_after;
        self
    }

    pub fn build(self) -> HealthConfig {
        self.config
    }
}

/// State synchronization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub interval: Duration,
    pub batch_size: usize,
    pub include_tunnel_state: bool,
    pub include_project_state: bool,
    pub include_server_state: bool,
    pub include_configuration: bool,
    pub include_statistics: bool,
    pub include_log_cursor: bool,
}

impl SyncConfig {
    pub fn builder() -> SyncConfigBuilder {
        SyncConfigBuilder::default()
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            batch_size: 1024,
            include_tunnel_state: true,
            include_project_state: true,
            include_server_state: true,
            include_configuration: true,
            include_statistics: true,
            include_log_cursor: false,
        }
    }
}

/// Builder for [`SyncConfig`].
#[derive(Debug, Clone)]
pub struct SyncConfigBuilder {
    config: SyncConfig,
}

impl Default for SyncConfigBuilder {
    fn default() -> Self {
        Self {
            config: SyncConfig::default(),
        }
    }
}

impl SyncConfigBuilder {
    pub fn interval(mut self, interval: Duration) -> Self {
        self.config.interval = interval;
        self
    }

    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.config.batch_size = batch_size;
        self
    }

    pub fn include_tunnel_state(mut self, enabled: bool) -> Self {
        self.config.include_tunnel_state = enabled;
        self
    }

    pub fn include_project_state(mut self, enabled: bool) -> Self {
        self.config.include_project_state = enabled;
        self
    }

    pub fn include_server_state(mut self, enabled: bool) -> Self {
        self.config.include_server_state = enabled;
        self
    }

    pub fn include_configuration(mut self, enabled: bool) -> Self {
        self.config.include_configuration = enabled;
        self
    }

    pub fn include_statistics(mut self, enabled: bool) -> Self {
        self.config.include_statistics = enabled;
        self
    }

    pub fn include_log_cursor(mut self, enabled: bool) -> Self {
        self.config.include_log_cursor = enabled;
        self
    }

    pub fn build(self) -> SyncConfig {
        self.config
    }
}
