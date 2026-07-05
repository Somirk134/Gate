//! Health checking interfaces.

use crate::connection::Connection;
use crate::core::Tunnel;
use crate::forwarder::Forwarder;
use crate::listener::Listener;
use crate::runtime::RuntimeManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub component: String,
    pub message: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct HealthChecker;

impl HealthChecker {
    pub fn check_tunnel(&self, tunnel: &dyn Tunnel) -> HealthReport {
        HealthReport {
            status: tunnel.health(),
            component: "tunnel".to_string(),
            message: None,
        }
    }

    pub fn check_connection(&self, _connection: &Connection) -> HealthReport {
        HealthReport {
            status: HealthStatus::Unknown,
            component: "connection".to_string(),
            message: None,
        }
    }

    pub fn check_listener(&self, _listener: &dyn Listener) -> HealthReport {
        HealthReport {
            status: HealthStatus::Unknown,
            component: "listener".to_string(),
            message: None,
        }
    }

    pub fn check_forwarder(&self, _forwarder: &dyn Forwarder) -> HealthReport {
        HealthReport {
            status: HealthStatus::Unknown,
            component: "forwarder".to_string(),
            message: None,
        }
    }

    pub fn check_runtime(&self, _runtime: &RuntimeManager) -> HealthReport {
        HealthReport {
            status: HealthStatus::Unknown,
            component: "runtime".to_string(),
            message: None,
        }
    }
}
