//! Engine core: context, lifecycle, state, manager, builder, and tunnel trait.

mod builder;
mod config;
mod context;
mod lifecycle;
mod manager;
mod state;
mod tunnel;
mod tunnel_engine;

pub use builder::EngineBuilder;
pub use config::{EngineConfig, EngineConfigBuilder};
pub use context::EngineContext;
pub use lifecycle::EngineLifecycle;
pub use manager::EngineManager;
pub use state::{EnginePhase, EngineState};
pub use tunnel::{Tunnel, TunnelId, TunnelStatus};
pub use tunnel_engine::TunnelEngine;
