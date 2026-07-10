use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::aggregator::{AggregationPeriod, Aggregator, StatisticsAggregator};
use super::alert::{AlertKind, AlertManager, AlertRule, AlertSeverity};
use super::collector::{Collector, CollectorRegistry};
use super::config::StatisticsConfig;
use super::dashboard::{
    ConnectionTrendPoint, DashboardData, DashboardProvider, OverviewStatistics, RealtimeSpeedPoint,
    RecentActivity, ServerStatusSummary, StatusBucket, TrafficTrendPoint,
};
use super::error::StatisticsResult;
use super::event::StatisticsEvent;
use super::health::{
    HealthCenter, HealthProvider, HealthReport, HealthSignal, HealthStatus, HealthTarget,
};
use super::sampler::{Sampler, SamplingStrategy, StatisticsSampler};
use super::statistics::Statistics;

/// Monitoring center lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonitorLifecycleState {
    Created,
    Running,
    Stopped,
}

/// Unified monitoring center facade.
pub struct MonitoringCenter {
    config: StatisticsConfig,
    state: MonitorLifecycleState,
    collectors: CollectorRegistry,
    aggregator: StatisticsAggregator,
    sampler: StatisticsSampler,
    health: HealthCenter,
    alerts: AlertManager,
    events: Vec<StatisticsEvent>,
}

impl MonitoringCenter {
    /// Creates a monitoring center with default dependencies.
    pub fn new(config: StatisticsConfig) -> Self {
        let strategy = SamplingStrategy {
            interval: config.sampling.interval,
            max_samples: config.sampling.max_samples,
            retain_history: false,
        };
        let alerts = AlertManager::new(vec![
            AlertRule::new(
                "cpu-high",
                AlertKind::CpuHigh,
                AlertSeverity::Warning,
                config.alert.cpu_high_percent,
            ),
            AlertRule::new(
                "memory-high",
                AlertKind::MemoryHigh,
                AlertSeverity::Warning,
                config.alert.memory_high_percent,
            ),
            AlertRule::new(
                "traffic-overflow",
                AlertKind::TrafficOverflow,
                AlertSeverity::Critical,
                config.alert.traffic_overflow_bytes as f64,
            ),
        ]);

        Self {
            config,
            state: MonitorLifecycleState::Created,
            collectors: CollectorRegistry::new(),
            aggregator: StatisticsAggregator::default(),
            sampler: StatisticsSampler::new(strategy),
            health: HealthCenter::new(),
            alerts,
            events: Vec::new(),
        }
    }

    /// Starts the monitoring center lifecycle.
    pub fn start(&mut self) {
        tracing::info!(target: "gate::statistics", "Collector Start");
        self.state = MonitorLifecycleState::Running;
    }

    /// Stops the monitoring center lifecycle.
    pub fn stop(&mut self) {
        tracing::info!(target: "gate::statistics", "Collector Stop");
        self.state = MonitorLifecycleState::Stopped;
    }

    /// Returns the current lifecycle state.
    pub fn state(&self) -> MonitorLifecycleState {
        self.state
    }

    /// Returns immutable configuration.
    pub fn config(&self) -> &StatisticsConfig {
        &self.config
    }

    /// Registers a collector.
    pub fn register_collector(&mut self, collector: Box<dyn Collector>) -> StatisticsResult<()> {
        let name = collector.name().to_string();
        self.collectors.register(collector)?;
        self.events.push(StatisticsEvent::CollectorRegistered {
            collector: name,
            timestamp: Utc::now(),
        });
        Ok(())
    }

    /// Removes a collector.
    pub fn remove_collector(&mut self, name: &str) -> StatisticsResult<()> {
        self.collectors.remove(name)?;
        self.events.push(StatisticsEvent::CollectorRemoved {
            collector: name.to_string(),
            timestamp: Utc::now(),
        });
        Ok(())
    }

    /// Collects metrics, aggregates statistics, samples, evaluates health, and emits events.
    pub fn collect_once(&mut self) -> StatisticsResult<Statistics> {
        let metrics = self.collectors.collect_all()?;
        self.events
            .push(StatisticsEvent::metrics_collected("all", metrics.clone()));

        let statistics = self.aggregator.ingest_metrics(metrics)?;
        self.sampler.sample(statistics.clone())?;
        self.update_health_from_statistics(&statistics);
        for event in self.alerts.evaluate_statistics(&statistics) {
            self.events.push(StatisticsEvent::alert_triggered(event));
        }
        self.events
            .push(StatisticsEvent::statistics_updated(statistics.clone()));
        Ok(statistics)
    }

    /// Returns latest statistics.
    pub fn statistics(&self) -> Statistics {
        self.aggregator
            .snapshot(AggregationPeriod::Realtime)
            .unwrap_or_else(Statistics::default)
    }

    /// Returns current health report.
    pub fn health_report(&self) -> HealthReport {
        self.health.report()
    }

    /// Returns emitted events.
    pub fn events(&self) -> &[StatisticsEvent] {
        &self.events
    }

    fn update_health_from_statistics(&mut self, statistics: &Statistics) {
        let system_status = if statistics.system.cpu_usage >= 90.0
            || statistics.system.memory_usage >= 90.0
        {
            HealthStatus::Critical
        } else if statistics.system.cpu_usage >= 75.0 || statistics.system.memory_usage >= 80.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        self.health.update(HealthSignal::new(
            HealthTarget::System,
            system_status,
            "System resource health",
            100.0
                - statistics
                    .system
                    .cpu_usage
                    .max(statistics.system.memory_usage),
        ));

        let connection_status = if statistics.connection.failure > statistics.connection.success {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        self.health.update(HealthSignal::new(
            HealthTarget::Connection,
            connection_status,
            "Connection health",
            100.0,
        ));
        self.events
            .push(StatisticsEvent::health_changed(self.health.report()));
    }

    fn build_realtime_speed(statistics: &Statistics) -> Vec<RealtimeSpeedPoint> {
        vec![RealtimeSpeedPoint {
            timestamp: Utc::now(),
            upload_bps: statistics.network.traffic.upload_speed_bps,
            download_bps: statistics.network.traffic.download_speed_bps,
        }]
    }
}

impl Default for MonitoringCenter {
    fn default() -> Self {
        Self::new(StatisticsConfig::default())
    }
}

impl DashboardProvider for MonitoringCenter {
    fn dashboard_data(&self) -> DashboardData {
        let statistics = self.statistics();
        let health = self.health.report();
        DashboardData {
            overview: OverviewStatistics {
                tunnel_count: statistics.tunnel.tunnel_count,
                running_tunnel: statistics.tunnel.running_tunnel,
                current_connection: statistics.connection.current_connection,
                today_traffic: statistics.tunnel.today_traffic,
                total_traffic: statistics.tunnel.total_traffic,
                average_rtt_ms: statistics.connection.average_rtt_ms,
                runtime_uptime_seconds: statistics.runtime.runtime_uptime_seconds,
                health_score: health
                    .signals
                    .iter()
                    .map(|signal| signal.score)
                    .reduce(f64::min)
                    .unwrap_or(100.0),
            },
            traffic: statistics.network.traffic.clone(),
            realtime_speed: Self::build_realtime_speed(&statistics),
            tunnel_status: vec![
                StatusBucket {
                    label: "running".to_string(),
                    count: statistics.tunnel.running_tunnel,
                },
                StatusBucket {
                    label: "stopped".to_string(),
                    count: statistics.tunnel.stopped_tunnel,
                },
            ],
            tunnel: statistics.tunnel.clone(),
            server_status: ServerStatusSummary {
                online: statistics.server.online_server,
                warning: statistics.server.warning_server,
                offline: statistics.server.offline_server,
            },
            system_health: health,
            system: statistics.system.clone(),
            connection: statistics.connection.clone(),
            connection_trend: vec![ConnectionTrendPoint {
                timestamp: Utc::now(),
                current: statistics.connection.current_connection,
                success: statistics.connection.success,
                failure: statistics.connection.failure,
                reconnect: statistics.connection.reconnect,
            }],
            traffic_trend: vec![TrafficTrendPoint {
                timestamp: Utc::now(),
                upload_bytes: statistics.network.traffic.upload_bytes,
                download_bytes: statistics.network.traffic.download_bytes,
            }],
            runtime: statistics.runtime.clone(),
            recent_activity: vec![RecentActivity {
                id: "monitoring-center-ready".to_string(),
                title: "Monitoring Center ready".to_string(),
                category: "system".to_string(),
                timestamp: Utc::now(),
            }],
            generated_at: Utc::now(),
        }
    }
}

impl HealthProvider for MonitoringCenter {
    fn health_signal(&self) -> HealthSignal {
        let report = self.health.report();
        HealthSignal::new(
            HealthTarget::System,
            report.overall,
            "Monitoring center health",
            100.0,
        )
    }
}
