//! Independent TLS infrastructure for Gate server.
//!
//! This crate provides the TLS, ACME, certificate store, and renewal primitives
//! used by the server-side HTTPS integration.

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
