//! TCP connector with retry, timeout, and connection state.

use crate::runtime::config::{ConnectorConfig, RetryConfig, TimeoutConfig};
use crate::runtime::error::ConnectorError;
use crate::runtime::monitor::TrafficStatistics;
use crate::runtime::state::ConnectionState;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

#[derive(Debug)]
struct TcpConnectorInner {
    config: ConnectorConfig,
    retry: RetryConfig,
    timeout: TimeoutConfig,
    statistics: Arc<TrafficStatistics>,
    state: RwLock<ConnectionState>,
}

/// TCP connector for target services.
#[derive(Debug, Clone)]
pub struct TcpConnector {
    inner: Arc<TcpConnectorInner>,
}

impl TcpConnector {
    pub fn new(
        config: ConnectorConfig,
        retry: RetryConfig,
        timeout: TimeoutConfig,
        statistics: Arc<TrafficStatistics>,
    ) -> Self {
        Self {
            inner: Arc::new(TcpConnectorInner {
                config,
                retry,
                timeout,
                statistics,
                state: RwLock::new(ConnectionState::Created),
            }),
        }
    }

    pub fn state(&self) -> ConnectionState {
        *self.inner.state.read()
    }

    pub fn target_addr(&self) -> std::net::SocketAddr {
        self.inner.config.target_addr
    }

    pub async fn connect(&self) -> Result<TcpStream, ConnectorError> {
        self.connect_with_state(ConnectionState::Connecting).await
    }

    pub async fn reconnect(&self) -> Result<TcpStream, ConnectorError> {
        self.inner.statistics.increment_reconnect();
        self.connect_with_state(ConnectionState::Reconnecting).await
    }

    async fn connect_with_state(
        &self,
        initial_state: ConnectionState,
    ) -> Result<TcpStream, ConnectorError> {
        self.set_state(initial_state);
        let attempts = self.inner.retry.max_attempts.max(1);
        let target_addr = self.inner.config.target_addr;
        let mut last_error = None;

        for attempt in 1..=attempts {
            if attempt > 1 {
                self.set_state(ConnectionState::Retrying);
                self.inner.statistics.increment_reconnect();
            }

            match timeout(
                self.inner.timeout.connect_timeout,
                TcpStream::connect(target_addr),
            )
            .await
            {
                Ok(Ok(stream)) => {
                    if self.inner.config.tcp_nodelay {
                        if let Err(source) = stream.set_nodelay(true) {
                            self.inner.statistics.increment_error();
                            self.set_state(ConnectionState::Failed);
                            return Err(ConnectorError::Connect {
                                addr: target_addr,
                                source,
                            });
                        }
                    }

                    self.set_state(ConnectionState::Connected);
                    info!(
                        target: "gate_runtime",
                        addr = %target_addr,
                        attempt,
                        "Connect Success"
                    );
                    return Ok(stream);
                }
                Ok(Err(source)) => {
                    self.inner.statistics.increment_error();
                    warn!(
                        target: "gate_runtime",
                        addr = %target_addr,
                        attempt,
                        error = %source,
                        "Connect Failed"
                    );
                    last_error = Some(ConnectorError::Connect {
                        addr: target_addr,
                        source,
                    });
                }
                Err(_) => {
                    self.inner.statistics.increment_error();
                    warn!(
                        target: "gate_runtime",
                        addr = %target_addr,
                        attempt,
                        timeout = ?self.inner.timeout.connect_timeout,
                        "Connect Failed"
                    );
                    last_error = Some(ConnectorError::ConnectTimeout {
                        addr: target_addr,
                        timeout: self.inner.timeout.connect_timeout,
                    });
                }
            }

            if attempt < attempts {
                sleep(self.inner.retry.delay_for_attempt(attempt)).await;
            }
        }

        self.set_state(ConnectionState::Failed);
        Err(last_error.unwrap_or(ConnectorError::RetryExhausted {
            addr: target_addr,
            attempts,
        }))
    }

    fn set_state(&self, state: ConnectionState) {
        *self.inner.state.write() = state;
    }
}
