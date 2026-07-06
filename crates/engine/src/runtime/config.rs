//! Runtime configuration and builders.

use crate::core::TunnelId;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

/// Backoff calculation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Linear,
    Exponential,
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential
    }
}

/// TCP listener configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    pub tunnel_id: TunnelId,
    pub listen_addr: SocketAddr,
    pub tcp_nodelay: bool,
    pub accept_backoff: Duration,
    pub max_connections: usize,
}

impl ListenerConfig {
    pub fn builder() -> ListenerConfigBuilder {
        ListenerConfigBuilder::default()
    }
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            tunnel_id: TunnelId::new(),
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            tcp_nodelay: true,
            accept_backoff: Duration::from_millis(100),
            max_connections: 4096,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListenerConfigBuilder {
    config: ListenerConfig,
}

impl Default for ListenerConfigBuilder {
    fn default() -> Self {
        Self {
            config: ListenerConfig::default(),
        }
    }
}

impl ListenerConfigBuilder {
    pub fn tunnel_id(mut self, tunnel_id: TunnelId) -> Self {
        self.config.tunnel_id = tunnel_id;
        self
    }

    pub fn listen_addr(mut self, listen_addr: SocketAddr) -> Self {
        self.config.listen_addr = listen_addr;
        self
    }

    pub fn tcp_nodelay(mut self, tcp_nodelay: bool) -> Self {
        self.config.tcp_nodelay = tcp_nodelay;
        self
    }

    pub fn accept_backoff(mut self, accept_backoff: Duration) -> Self {
        self.config.accept_backoff = accept_backoff;
        self
    }

    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.config.max_connections = max_connections;
        self
    }

    pub fn build(self) -> ListenerConfig {
        self.config
    }
}

/// TCP connector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub target_addr: SocketAddr,
    pub tcp_nodelay: bool,
    pub keepalive: Option<Duration>,
}

impl ConnectorConfig {
    pub fn builder() -> ConnectorConfigBuilder {
        ConnectorConfigBuilder::default()
    }
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            target_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            tcp_nodelay: true,
            keepalive: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectorConfigBuilder {
    config: ConnectorConfig,
}

impl Default for ConnectorConfigBuilder {
    fn default() -> Self {
        Self {
            config: ConnectorConfig::default(),
        }
    }
}

impl ConnectorConfigBuilder {
    pub fn target_addr(mut self, target_addr: SocketAddr) -> Self {
        self.config.target_addr = target_addr;
        self
    }

    pub fn tcp_nodelay(mut self, tcp_nodelay: bool) -> Self {
        self.config.tcp_nodelay = tcp_nodelay;
        self
    }

    pub fn keepalive(mut self, keepalive: Option<Duration>) -> Self {
        self.config.keepalive = keepalive;
        self
    }

    pub fn build(self) -> ConnectorConfig {
        self.config
    }
}

/// Buffer pool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    pub fixed_buffer_size: usize,
    pub dynamic_buffer_limit: usize,
    pub pool_capacity: usize,
}

impl BufferConfig {
    pub fn builder() -> BufferConfigBuilder {
        BufferConfigBuilder::default()
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            fixed_buffer_size: 64 * 1024,
            dynamic_buffer_limit: 1024 * 1024,
            pool_capacity: 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferConfigBuilder {
    config: BufferConfig,
}

impl Default for BufferConfigBuilder {
    fn default() -> Self {
        Self {
            config: BufferConfig::default(),
        }
    }
}

impl BufferConfigBuilder {
    pub fn fixed_buffer_size(mut self, fixed_buffer_size: usize) -> Self {
        self.config.fixed_buffer_size = fixed_buffer_size;
        self
    }

    pub fn dynamic_buffer_limit(mut self, dynamic_buffer_limit: usize) -> Self {
        self.config.dynamic_buffer_limit = dynamic_buffer_limit;
        self
    }

    pub fn pool_capacity(mut self, pool_capacity: usize) -> Self {
        self.config.pool_capacity = pool_capacity;
        self
    }

    pub fn build(self) -> BufferConfig {
        self.config
    }
}

/// Retry and reconnect policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub strategy: BackoffStrategy,
    pub reconnect_enabled: bool,
}

impl RetryConfig {
    pub fn builder() -> RetryConfigBuilder {
        RetryConfigBuilder::default()
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let attempt = attempt.max(1) as u128;
        let base_ms = self.initial_delay.as_millis();
        let multiplier = match self.strategy {
            BackoffStrategy::Linear => attempt,
            BackoffStrategy::Exponential => 2_u128.saturating_pow((attempt - 1) as u32),
        };
        let delay_ms = base_ms.saturating_mul(multiplier);
        let capped_ms = delay_ms.min(self.max_delay.as_millis());
        Duration::from_millis(capped_ms.min(u64::MAX as u128) as u64)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(3),
            strategy: BackoffStrategy::Exponential,
            reconnect_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfigBuilder {
    config: RetryConfig,
}

impl Default for RetryConfigBuilder {
    fn default() -> Self {
        Self {
            config: RetryConfig::default(),
        }
    }
}

impl RetryConfigBuilder {
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.config.max_attempts = max_attempts;
        self
    }

    pub fn initial_delay(mut self, initial_delay: Duration) -> Self {
        self.config.initial_delay = initial_delay;
        self
    }

    pub fn max_delay(mut self, max_delay: Duration) -> Self {
        self.config.max_delay = max_delay;
        self
    }

    pub fn strategy(mut self, strategy: BackoffStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    pub fn reconnect_enabled(mut self, reconnect_enabled: bool) -> Self {
        self.config.reconnect_enabled = reconnect_enabled;
        self
    }

    pub fn build(self) -> RetryConfig {
        self.config
    }
}

/// Runtime timeout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub idle_timeout: Duration,
    pub connect_timeout: Duration,
    pub shutdown_timeout: Duration,
}

impl TimeoutConfig {
    pub fn builder() -> TimeoutConfigBuilder {
        TimeoutConfigBuilder::default()
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            connect_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeoutConfigBuilder {
    config: TimeoutConfig,
}

impl Default for TimeoutConfigBuilder {
    fn default() -> Self {
        Self {
            config: TimeoutConfig::default(),
        }
    }
}

impl TimeoutConfigBuilder {
    pub fn read_timeout(mut self, read_timeout: Duration) -> Self {
        self.config.read_timeout = read_timeout;
        self
    }

    pub fn write_timeout(mut self, write_timeout: Duration) -> Self {
        self.config.write_timeout = write_timeout;
        self
    }

    pub fn idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.config.idle_timeout = idle_timeout;
        self
    }

    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.config.connect_timeout = connect_timeout;
        self
    }

    pub fn shutdown_timeout(mut self, shutdown_timeout: Duration) -> Self {
        self.config.shutdown_timeout = shutdown_timeout;
        self
    }

    pub fn build(self) -> TimeoutConfig {
        self.config
    }
}

/// Single TCP tunnel runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub name: String,
    pub listener: ListenerConfig,
    pub connector: ConnectorConfig,
    pub buffer: BufferConfig,
    pub retry: RetryConfig,
    pub timeout: TimeoutConfig,
    pub worker_threads: usize,
    pub max_tasks: usize,
    pub max_sessions: usize,
    pub monitor_interval: Duration,
    pub cleanup_interval: Duration,
}

impl RuntimeConfig {
    pub fn builder() -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::default()
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            name: "gate-runtime".to_string(),
            listener: ListenerConfig::default(),
            connector: ConnectorConfig::default(),
            buffer: BufferConfig::default(),
            retry: RetryConfig::default(),
            timeout: TimeoutConfig::default(),
            worker_threads: 4,
            max_tasks: 4096,
            max_sessions: 4096,
            monitor_interval: Duration::from_secs(1),
            cleanup_interval: Duration::from_secs(30),
        }
    }
}

impl From<crate::config::RuntimeConfig> for RuntimeConfig {
    fn from(value: crate::config::RuntimeConfig) -> Self {
        Self {
            worker_threads: value.worker_threads,
            max_tasks: value.max_tasks,
            max_sessions: value.max_tasks,
            timeout: TimeoutConfig {
                shutdown_timeout: value.shutdown_timeout,
                ..TimeoutConfig::default()
            },
            ..RuntimeConfig::default()
        }
    }
}

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
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    pub fn tunnel_id(mut self, tunnel_id: TunnelId) -> Self {
        self.config.listener.tunnel_id = tunnel_id;
        self
    }

    pub fn listen_addr(mut self, listen_addr: SocketAddr) -> Self {
        self.config.listener.listen_addr = listen_addr;
        self
    }

    pub fn target_addr(mut self, target_addr: SocketAddr) -> Self {
        self.config.connector.target_addr = target_addr;
        self
    }

    pub fn listener(mut self, listener: ListenerConfig) -> Self {
        self.config.listener = listener;
        self
    }

    pub fn connector(mut self, connector: ConnectorConfig) -> Self {
        self.config.connector = connector;
        self
    }

    pub fn buffer(mut self, buffer: BufferConfig) -> Self {
        self.config.buffer = buffer;
        self
    }

    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.config.retry = retry;
        self
    }

    pub fn timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn worker_threads(mut self, worker_threads: usize) -> Self {
        self.config.worker_threads = worker_threads;
        self
    }

    pub fn max_tasks(mut self, max_tasks: usize) -> Self {
        self.config.max_tasks = max_tasks;
        self
    }

    pub fn max_sessions(mut self, max_sessions: usize) -> Self {
        self.config.max_sessions = max_sessions;
        self
    }

    pub fn monitor_interval(mut self, monitor_interval: Duration) -> Self {
        self.config.monitor_interval = monitor_interval;
        self
    }

    pub fn cleanup_interval(mut self, cleanup_interval: Duration) -> Self {
        self.config.cleanup_interval = cleanup_interval;
        self
    }

    pub fn build(self) -> RuntimeConfig {
        self.config
    }
}
