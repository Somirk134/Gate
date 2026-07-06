//! Domain Management Infrastructure.
//!
//! This module is intentionally independent from the existing server runtime,
//! tunnel runtime, HTTP runtime, TLS stack, communication layer, dashboard, and
//! persistent database schema.
//!
//! The boundary is designed for later integration by injection:
//!
//! - HTTP and HTTPS runtimes can depend only on [`resolver::HostResolver`].
//! - TLS, ACME, certificate stores, and DNS providers can depend only on traits
//!   from [`traits`] and [`resolver::DnsResolver`].
//! - Repository implementations can be swapped behind [`repository::DomainRepository`].
//!
//! The default implementation is in-memory only. It does not open sockets, query
//! DNS, touch runtime state, or require Cargo workspace changes.

pub mod config;
pub mod error;
pub mod event;
pub mod model;
pub mod repository;
pub mod resolver;
pub mod service;
pub mod storage;
pub mod traits;
pub mod validator;

#[cfg(test)]
mod tests;
