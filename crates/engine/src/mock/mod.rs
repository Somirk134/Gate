//! Stability mocks for heartbeat, reconnect, recovery, and health tests.
//!
//! These mocks simulate failure modes without opening sockets or restoring
//! business data.

use crate::config::{HealthConfig, HeartbeatConfig, ReconnectConfig, ReconnectStrategyConfig};
use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::health::{HealthManager, HealthReport, HealthSignal};
use crate::heartbeat::{HeartbeatManager, HeartbeatSnapshot};
use crate::reconnect::{ReconnectManager, ReconnectRequest, ReconnectSnapshot};
use crate::session::SessionId;
use crate::session_recovery::{RecoveryContext, RecoveryResult, SessionRecoveryManager};
use crate::statistics::StabilityMetrics;
use std::time::Duration;

/// Built-in mock scenario catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockScenario {
    Timeout,
    Disconnected,
    Recovered,
    NetworkJitter,
    ConsecutiveFailures(u32),
    SuccessfulRecovery,
}

/// Mock heartbeat driver.
pub struct MockHeartbeat {
    manager: HeartbeatManager,
}

impl Default for MockHeartbeat {
    fn default() -> Self {
        Self {
            manager: HeartbeatManager::new(
                HeartbeatConfig::builder()
                    .interval(Duration::from_millis(50))
                    .timeout(Duration::from_millis(50))
                    .retry_count(2)
                    .retry_delay(Duration::from_millis(10))
                    .max_missed_heartbeat(2)
                    .build(),
            ),
        }
    }
}

impl MockHeartbeat {
    pub fn manager(&self) -> &HeartbeatManager {
        &self.manager
    }

    pub async fn simulate_timeout(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<HeartbeatSnapshot, crate::error::HeartbeatError> {
        self.manager.start(tunnel_id).await?;
        self.manager.ping(tunnel_id).await?;
        Ok(self.manager.timeout(tunnel_id).await?)
    }

    pub async fn simulate_network_jitter(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<HeartbeatSnapshot, crate::error::HeartbeatError> {
        self.manager.start(tunnel_id).await?;
        let ping = self.manager.ping(tunnel_id).await?;
        Ok(self.manager.pong(tunnel_id, ping.sequence).await?)
    }
}

/// Mock reconnect driver.
pub struct MockReconnect {
    manager: ReconnectManager,
}

impl Default for MockReconnect {
    fn default() -> Self {
        Self {
            manager: ReconnectManager::new(
                ReconnectConfig::builder()
                    .max_attempts(5)
                    .strategy(ReconnectStrategyConfig::FixedInterval {
                        interval: Duration::from_millis(1),
                    })
                    .build(),
            ),
        }
    }
}

impl MockReconnect {
    pub fn manager(&self) -> &ReconnectManager {
        &self.manager
    }

    pub async fn simulate_disconnected(
        &self,
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
    ) -> Result<Option<ReconnectRequest>, crate::error::ReconnectError> {
        self.manager
            .auto_reconnect(tunnel_id, Some(connection_id), "mock disconnected")
            .await?;
        self.manager.schedule_next().await
    }

    pub async fn simulate_consecutive_failures(
        &self,
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
        failures: u32,
    ) -> Result<ReconnectSnapshot, crate::error::ReconnectError> {
        self.manager
            .auto_reconnect(tunnel_id, Some(connection_id), "mock failures")
            .await?;
        let mut snapshot = self.manager.snapshot(tunnel_id)?;

        for attempt in 0..failures {
            let _ = self.manager.schedule_next().await?;
            snapshot = self
                .manager
                .mark_failed(tunnel_id, format!("mock failure {attempt}"))
                .await?;
            if attempt + 1 < failures {
                self.manager
                    .auto_reconnect(tunnel_id, Some(connection_id), "retry mock failure")
                    .await?;
            }
        }

        Ok(snapshot)
    }

    pub async fn simulate_successful_recovery(
        &self,
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
    ) -> Result<ReconnectSnapshot, crate::error::ReconnectError> {
        self.manager
            .manual_reconnect(tunnel_id, Some(connection_id), "mock manual recovery")
            .await?;
        let _ = self.manager.schedule_next().await?;
        self.manager.mark_succeeded(tunnel_id).await
    }
}

/// Mock recovery driver.
#[derive(Default)]
pub struct MockRecovery {
    manager: SessionRecoveryManager,
}

impl MockRecovery {
    pub fn manager(&self) -> &SessionRecoveryManager {
        &self.manager
    }

    pub async fn simulate_recovered(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<RecoveryResult, crate::error::RecoveryError> {
        self.manager
            .capture(
                RecoveryContext::new(tunnel_id)
                    .session_id(SessionId::new())
                    .statistics(StabilityMetrics::default())
                    .subscription("events", Some("0".to_string()))
                    .attribute("mock", "true"),
            )
            .await;
        self.manager.recover_all(tunnel_id).await
    }
}

/// Mock health driver.
pub struct MockHealth {
    manager: HealthManager,
}

impl Default for MockHealth {
    fn default() -> Self {
        Self {
            manager: HealthManager::new(HealthConfig::default()),
        }
    }
}

impl MockHealth {
    pub fn manager(&self) -> &HealthManager {
        &self.manager
    }

    pub async fn simulate_signal(&self, signal: HealthSignal) -> HealthReport {
        self.manager.record_signal(signal).await
    }
}
