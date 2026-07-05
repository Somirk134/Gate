//! Shared communication primitives that are not tied to client or server role.

use futures::future::BoxFuture;

pub mod logging;

pub use logging::CommunicationLogger;

/// Boxed asynchronous result used by communication traits.
pub type CommunicationFuture<'a, T> = BoxFuture<'a, crate::error::CommunicationResult<T>>;
