use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::health::{HealthReport, HealthStatus, HealthTarget};
use super::statistics::Statistics;

/// Alert event kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertKind {
    CpuHigh,
    MemoryHigh,
    ConnectionLost,
    HeartbeatTimeout,
    TrafficOverflow,
    Custom(String),
}

/// Alert severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert rule descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub kind: AlertKind,
    pub severity: AlertSeverity,
    pub threshold: f64,
    pub enabled: bool,
}

impl AlertRule {
    /// Creates an alert rule.
    pub fn new(
        id: impl Into<String>,
        kind: AlertKind,
        severity: AlertSeverity,
        threshold: f64,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            severity,
            threshold,
            enabled: true,
        }
    }
}

/// Alert event emitted by the monitoring center.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub kind: AlertKind,
    pub severity: AlertSeverity,
    pub message: String,
    pub target: Option<HealthTarget>,
    pub value: f64,
    pub threshold: f64,
    pub triggered_at: DateTime<Utc>,
}

/// Alert manager with in-memory rules and events.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AlertManager {
    rules: Vec<AlertRule>,
    events: Vec<AlertEvent>,
}

impl AlertManager {
    /// Creates an alert manager.
    pub fn new(rules: Vec<AlertRule>) -> Self {
        Self {
            rules,
            events: Vec::new(),
        }
    }

    /// Adds a rule.
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Returns configured rules.
    pub fn rules(&self) -> &[AlertRule] {
        &self.rules
    }

    /// Returns emitted alert events.
    pub fn events(&self) -> &[AlertEvent] {
        &self.events
    }

    /// Evaluates statistics against configured rules.
    pub fn evaluate_statistics(&mut self, statistics: &Statistics) -> Vec<AlertEvent> {
        let mut emitted = Vec::new();
        for rule in self.rules.iter().filter(|rule| rule.enabled) {
            let value = match rule.kind {
                AlertKind::CpuHigh => statistics.system.cpu_usage,
                AlertKind::MemoryHigh => statistics.system.memory_usage,
                AlertKind::TrafficOverflow => statistics.network.traffic.total_traffic_bytes as f64,
                AlertKind::ConnectionLost => statistics.connection.failure as f64,
                AlertKind::HeartbeatTimeout => statistics.heartbeat.timeout_count as f64,
                AlertKind::Custom(_) => continue,
            };

            if value >= rule.threshold {
                let event = AlertEvent {
                    id: format!("{}-{}", rule.id, Utc::now().timestamp_millis()),
                    kind: rule.kind.clone(),
                    severity: rule.severity,
                    message: format!("{:?} threshold exceeded", rule.kind),
                    target: None,
                    value,
                    threshold: rule.threshold,
                    triggered_at: Utc::now(),
                };
                tracing::warn!(target: "gate::statistics", alert = ?event.kind, "Alert triggered");
                emitted.push(event.clone());
                self.events.push(event);
            }
        }
        emitted
    }

    /// Converts unhealthy health states to alert events.
    pub fn evaluate_health(&mut self, report: &HealthReport) -> Vec<AlertEvent> {
        let mut emitted = Vec::new();
        for signal in &report.signals {
            if matches!(signal.status, HealthStatus::Critical | HealthStatus::Offline) {
                let event = AlertEvent {
                    id: format!("health-{}", Utc::now().timestamp_millis()),
                    kind: AlertKind::Custom("health.changed".to_string()),
                    severity: AlertSeverity::Critical,
                    message: signal.message.clone(),
                    target: Some(signal.target.clone()),
                    value: signal.score,
                    threshold: 0.0,
                    triggered_at: Utc::now(),
                };
                emitted.push(event.clone());
                self.events.push(event);
            }
        }
        emitted
    }

    /// Clears emitted events.
    pub fn reset(&mut self) {
        self.events.clear();
    }
}
