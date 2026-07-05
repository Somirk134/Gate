use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::{error::TransportError, packet::Packet};

pub type TransportFuture<'a, T> = BoxFuture<'a, Result<T, TransportError>>;

/// Transport endpoint description. No socket is opened by this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportEndpoint {
    Tcp { host: String, port: u16 },
    UnixSocket { path: String },
    NamedPipe { name: String },
    Custom(String),
}

/// Async-compatible transport boundary for future Tokio TCP adapters.
pub trait Transport: Send + Sync {
    fn name(&self) -> &'static str;

    fn connect<'a>(&'a mut self, endpoint: &'a TransportEndpoint) -> TransportFuture<'a, ()>;

    fn send<'a>(&'a mut self, packet: Packet) -> TransportFuture<'a, ()>;

    fn receive<'a>(&'a mut self) -> TransportFuture<'a, Packet>;

    fn close<'a>(&'a mut self) -> TransportFuture<'a, ()>;
}
