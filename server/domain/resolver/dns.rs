#[cfg(test)]
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::config::DomainConfig;
use crate::error::DnsError;
use crate::model::{DnsStatus, Host, RecordType};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DnsQuery {
    pub host: Host,
    pub record_type: RecordType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DnsAnswer {
    pub record_type: RecordType,
    pub ttl: u32,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DnsCheckResult {
    pub host: Host,
    pub record_type: RecordType,
    pub status: DnsStatus,
    pub ttl: u32,
    pub values: Vec<String>,
    pub resolved_to_server: bool,
}

pub trait DnsResolver: Send + Sync {
    fn resolve(&self, query: &DnsQuery) -> Result<DnsAnswer, DnsError>;
}

#[derive(Clone)]
pub struct DnsChecker<R>
where
    R: DnsResolver,
{
    resolver: R,
    config: DomainConfig,
}

impl<R> DnsChecker<R>
where
    R: DnsResolver,
{
    pub fn new(resolver: R, config: DomainConfig) -> Self {
        Self { resolver, config }
    }

    pub fn check(&self, query: &DnsQuery) -> Result<DnsCheckResult, DnsError> {
        let answer = self.resolver.resolve(query)?;
        let resolved_to_server = answer.values.iter().any(|value| {
            self.config
                .server_addresses
                .iter()
                .any(|expected| expected == value)
        });

        let status = if resolved_to_server {
            DnsStatus::Matched
        } else if answer.values.is_empty() {
            DnsStatus::NoRecord
        } else {
            DnsStatus::Mismatched
        };

        Ok(DnsCheckResult {
            host: query.host.clone(),
            record_type: query.record_type.clone(),
            ttl: answer.ttl,
            values: answer.values,
            resolved_to_server,
            status,
        })
    }
}

#[cfg(test)]
#[derive(Clone, Default)]
pub struct TestDnsResolver {
    records: Arc<RwLock<HashMap<DnsQuery, DnsAnswer>>>,
}

#[cfg(test)]
impl TestDnsResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_record(&self, query: DnsQuery, answer: DnsAnswer) -> Result<Self, DnsError> {
        let mut guard = self.records.write().map_err(|_| {
            DnsError::ResolverUnavailable("test DNS resolver lock poisoned".to_string())
        })?;
        guard.insert(query, answer);
        Ok(self.clone())
    }
}

#[cfg(test)]
impl DnsResolver for TestDnsResolver {
    fn resolve(&self, query: &DnsQuery) -> Result<DnsAnswer, DnsError> {
        let guard = self.records.read().map_err(|_| {
            DnsError::ResolverUnavailable("test DNS resolver lock poisoned".to_string())
        })?;

        guard
            .get(query)
            .cloned()
            .ok_or_else(|| DnsError::RecordNotFound(query.host.to_string()))
    }
}
