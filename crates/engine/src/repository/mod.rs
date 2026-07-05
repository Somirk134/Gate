//! Reserved repository boundaries.

use crate::config::TunnelConfig;
use crate::core::TunnelId;
use crate::error::EngineError;
use futures::future::BoxFuture;

/// Reserved tunnel persistence boundary. No implementation is provided in this phase.
pub trait TunnelRepository: Send + Sync {
    fn save(&self, config: TunnelConfig) -> BoxFuture<'static, Result<(), EngineError>>;

    fn find(&self, id: TunnelId) -> BoxFuture<'static, Result<Option<TunnelConfig>, EngineError>>;

    fn delete(&self, id: TunnelId) -> BoxFuture<'static, Result<(), EngineError>>;

    fn list(&self) -> BoxFuture<'static, Result<Vec<TunnelConfig>, EngineError>>;
}
