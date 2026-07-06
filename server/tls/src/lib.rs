//! Independent TLS infrastructure for Gate server.
//!
//! This crate intentionally does not connect to the server runtime, HTTP tunnel,
//! communication layer, dashboard, or existing services. It provides only the
//! abstractions and local primitives needed by a future HTTPS integration.

pub mod config;
pub mod error;
pub mod provider;

#[path = "../../acme/src/mod.rs"]
pub mod acme;
#[path = "../../cert_store/src/mod.rs"]
pub mod cert_store;
#[path = "../../certificate/src/mod.rs"]
pub mod certificate;
#[path = "../../crypto/src/mod.rs"]
pub mod crypto;
#[path = "../../renew/src/mod.rs"]
pub mod renew;

pub use config::{ChallengeType, TlsConfig};
pub use error::{AcmeError, CertificateError, TlsError, TlsInfrastructureError};
pub use provider::{CertificateMaterial, ExpireCheck, StoreBackedTlsProvider, TlsProvider};
