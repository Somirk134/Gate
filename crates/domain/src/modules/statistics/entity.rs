use serde::{Deserialize, Serialize};

/// Logical module that can be connected to the monitoring center.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonitoredModule {
    Tunnel,
    Project,
    Server,
    Connection,
    Heartbeat,
    Authentication,
    Runtime,
    System,
    Network,
    Client,
    Custom(String),
}

/// Registration descriptor for a monitored module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorRegistration {
    pub module: MonitoredModule,
    pub collector_name: String,
    pub enabled: bool,
}

impl MonitorRegistration {
    /// Creates an enabled registration descriptor.
    pub fn new(module: MonitoredModule, collector_name: impl Into<String>) -> Self {
        Self {
            module,
            collector_name: collector_name.into(),
            enabled: true,
        }
    }
}

/// Public alias for the central monitoring entity.
pub type StatisticsCenter = super::monitor::MonitoringCenter;
