#![forbid(unsafe_code)]

//! Alpha V1 integration harness.
//!
//! The harness connects the real protocol codec, TCP communication transport,
//! authentication handshake, heartbeat request/response, tunnel runtime,
//! runtime statistics, and graceful shutdown paths used by integration tests.

pub mod client;
pub mod communication;
pub mod performance;
pub mod protocol;
pub mod runtime;
pub mod server;

pub use client::{AlphaClient, AlphaClientState};
pub use communication::{AlphaStatistics, AuthSnapshot, HeartbeatSnapshot};
pub use performance::{ConnectionSimulationPlan, SimulationScale};
pub use runtime::{RuntimeHarness, RuntimeHarnessConfig};
pub use server::{AlphaServer, AlphaServerConfig, AlphaServerState};
