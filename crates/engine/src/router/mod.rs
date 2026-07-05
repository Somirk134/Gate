//! Tunnel routing and dispatch boundary.

use crate::config::ProtocolKind;
use crate::connection::Connection;
use crate::core::{Tunnel, TunnelId};
use crate::error::{EngineError, TunnelError};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    pub protocol: ProtocolKind,
    pub host: String,
    pub port: u16,
    pub path: Option<String>,
}

/// Tunnel lookup, route matching, and connection dispatch.
#[derive(Default)]
pub struct TunnelRouter {
    tunnels: DashMap<TunnelId, Arc<dyn Tunnel>>,
}

impl TunnelRouter {
    pub fn register(&self, tunnel: Arc<dyn Tunnel>) {
        self.tunnels.insert(tunnel.id(), tunnel);
    }

    pub fn unregister(&self, tunnel_id: &TunnelId) -> Option<Arc<dyn Tunnel>> {
        self.tunnels.remove(tunnel_id).map(|(_, tunnel)| tunnel)
    }

    pub fn find(&self, tunnel_id: &TunnelId) -> Result<Arc<dyn Tunnel>, TunnelError> {
        self.tunnels
            .get(tunnel_id)
            .map(|entry| Arc::clone(entry.value()))
            .ok_or(TunnelError::NotFound { id: *tunnel_id })
    }

    pub fn match_route(&self, _request: &RouteRequest) -> Option<Arc<dyn Tunnel>> {
        self.tunnels
            .iter()
            .next()
            .map(|entry| Arc::clone(entry.value()))
    }

    pub fn dispatch_connection(
        &self,
        _tunnel_id: TunnelId,
        _connection: Arc<Connection>,
    ) -> Result<(), EngineError> {
        Ok(())
    }
}
