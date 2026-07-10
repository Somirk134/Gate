//! Domain Management Infrastructure.
//!
//! This module is intentionally independent from the existing server runtime,
//! tunnel runtime, HTTP runtime, TLS stack, communication layer, dashboard, and
//! persistent database schema.
//!
//! The boundary is designed for later integration by injection:
//!
//! - HTTP and HTTPS runtimes can depend only on [`resolver::HostResolver`].
//! - TLS, ACME, certificate stores, and DNS providers can depend on explicit
//!   runtime adapters plus [`resolver::DnsResolver`].
//! - Repository implementations can be swapped behind [`repository::DomainRepository`].
//!
//! The release implementation persists managed domains in SQLite when the
//! default `sqlite` feature is enabled. The module does not open sockets or
//! touch runtime state by itself.

pub mod config;
pub mod error;
pub mod event;
pub mod model;
pub mod repository;
pub mod resolver;
pub mod service;
pub mod storage;
pub mod validator;

#[cfg(test)]
mod tests;
