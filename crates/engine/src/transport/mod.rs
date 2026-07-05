//! Protocol and connector extension points.

use crate::config::{ProtocolConfig, ProtocolKind};
use crate::connection::ConnectionContext;
use crate::error::{ConnectionError, ProtocolError};
use futures::future::BoxFuture;

/// Protocol abstraction for future HTTP, TCP, HTTPS, UDP, and P2P engines.
pub trait Protocol: Send + Sync {
    fn kind(&self) -> ProtocolKind;

    fn name(&self) -> &'static str;

    fn validate_config(&self, _config: &ProtocolConfig) -> Result<(), ProtocolError> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct HttpProtocol;

#[derive(Debug, Default, Clone, Copy)]
pub struct TcpProtocol;

#[derive(Debug, Default, Clone, Copy)]
pub struct HttpsProtocol;

#[derive(Debug, Default, Clone, Copy)]
pub struct UdpProtocol;

#[derive(Debug, Default, Clone, Copy)]
pub struct P2pProtocol;

impl Protocol for HttpProtocol {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Http
    }

    fn name(&self) -> &'static str {
        "http"
    }
}

impl Protocol for TcpProtocol {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Tcp
    }

    fn name(&self) -> &'static str {
        "tcp"
    }
}

impl Protocol for HttpsProtocol {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Https
    }

    fn name(&self) -> &'static str {
        "https"
    }
}

impl Protocol for UdpProtocol {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Udp
    }

    fn name(&self) -> &'static str {
        "udp"
    }
}

impl Protocol for P2pProtocol {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::P2p
    }

    fn name(&self) -> &'static str {
        "p2p"
    }
}

/// Connector abstraction for outbound tunnel targets.
pub trait Connector: Send + Sync {
    fn connect(
        &self,
        context: ConnectionContext,
    ) -> BoxFuture<'static, Result<(), ConnectionError>>;

    fn reconnect(
        &self,
        context: ConnectionContext,
    ) -> BoxFuture<'static, Result<(), ConnectionError>>;

    fn disconnect(
        &self,
        context: ConnectionContext,
    ) -> BoxFuture<'static, Result<(), ConnectionError>>;

    fn heartbeat(
        &self,
        context: ConnectionContext,
    ) -> BoxFuture<'static, Result<(), ConnectionError>>;
}
