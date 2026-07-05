//! Forwarder abstraction.

use crate::connection::ConnectionContext;
use crate::error::ForwardError;
use bytes::Bytes;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForwarderStatus {
    Created,
    Active,
    Paused,
    Closing,
    Closed,
    Error,
}

/// Data forwarding boundary.
pub trait Forwarder: Send + Sync {
    fn forward(
        &self,
        context: ConnectionContext,
        chunk: Bytes,
    ) -> BoxFuture<'static, Result<(), ForwardError>>;

    fn close(&self, context: ConnectionContext) -> BoxFuture<'static, Result<(), ForwardError>>;

    fn pause(&self) -> BoxFuture<'static, Result<(), ForwardError>>;

    fn resume(&self) -> BoxFuture<'static, Result<(), ForwardError>>;

    fn flush(&self) -> BoxFuture<'static, Result<(), ForwardError>>;

    fn status(&self) -> ForwarderStatus;
}
