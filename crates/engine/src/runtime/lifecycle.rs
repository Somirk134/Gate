//! Runtime lifecycle contract.

use crate::runtime::error::RuntimeError;
use crate::runtime::state::RuntimeState;
use futures::future::BoxFuture;

/// Lifecycle operations supported by the tunnel runtime.
pub trait RuntimeLifecycle: Send + Sync {
    fn state(&self) -> RuntimeState;

    fn start(&self) -> BoxFuture<'static, Result<(), RuntimeError>>;

    fn stop(&self) -> BoxFuture<'static, Result<(), RuntimeError>>;

    fn restart(&self) -> BoxFuture<'static, Result<(), RuntimeError>>;

    fn pause(&self) -> BoxFuture<'static, Result<(), RuntimeError>>;

    fn resume(&self) -> BoxFuture<'static, Result<(), RuntimeError>>;

    fn shutdown(&self) -> BoxFuture<'static, Result<(), RuntimeError>>;
}
