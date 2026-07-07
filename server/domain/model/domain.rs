use std::collections::BTreeSet;

use crate::error::{BindError, DomainError};
use crate::model::{
    Alias, BindStatus, DnsStatus, DomainId, DomainStatus, Host, RecordType, ResolveStatus,
    TunnelId, VerifyStatus,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Domain {
    id: DomainId,
    host: Host,
    aliases: Vec<Alias>,
    tunnel_id: Option<TunnelId>,
    record_type: RecordType,
    verify_status: VerifyStatus,
    bind_status: BindStatus,
    resolve_status: ResolveStatus,
    dns_status: DnsStatus,
    status: DomainStatus,
}

impl Domain {
    pub fn builder(id: DomainId, host: Host) -> DomainBuilder {
        DomainBuilder::new(id, host)
    }

    pub fn id(&self) -> &DomainId {
        &self.id
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn aliases(&self) -> &[Alias] {
        &self.aliases
    }

    pub fn tunnel_id(&self) -> Option<&TunnelId> {
        self.tunnel_id.as_ref()
    }

    pub fn record_type(&self) -> &RecordType {
        &self.record_type
    }

    pub fn verify_status(&self) -> &VerifyStatus {
        &self.verify_status
    }

    pub fn bind_status(&self) -> &BindStatus {
        &self.bind_status
    }

    pub fn resolve_status(&self) -> &ResolveStatus {
        &self.resolve_status
    }

    pub fn dns_status(&self) -> &DnsStatus {
        &self.dns_status
    }

    pub fn status(&self) -> &DomainStatus {
        &self.status
    }

    pub fn is_enabled(&self) -> bool {
        self.status != DomainStatus::Disabled && self.status != DomainStatus::Deleted
    }

    pub fn rename(&mut self, host: Host) {
        self.host = host;
        self.verify_status = VerifyStatus::Unverified;
        self.resolve_status = ResolveStatus::Unknown;
        self.dns_status = DnsStatus::NotChecked;
    }

    pub fn replace_aliases(&mut self, aliases: Vec<Alias>) -> Result<(), DomainError> {
        ensure_unique_aliases(&self.host, &aliases)?;
        self.aliases = aliases;
        Ok(())
    }

    pub fn update_record_type(&mut self, record_type: RecordType) {
        self.record_type = record_type;
        self.dns_status = DnsStatus::NotChecked;
    }

    pub fn bind(&mut self, tunnel_id: TunnelId) -> Result<(), BindError> {
        if self.status == DomainStatus::Disabled {
            return Err(BindError::DisabledDomain(self.id.to_string()));
        }

        if let Some(existing) = &self.tunnel_id {
            if existing == &tunnel_id {
                return Ok(());
            }
            return Err(BindError::AlreadyBound {
                domain_id: self.id.to_string(),
                tunnel_id: existing.to_string(),
            });
        }

        self.tunnel_id = Some(tunnel_id);
        self.bind_status = BindStatus::Bound;
        Ok(())
    }

    pub fn unbind(&mut self) -> Result<(), BindError> {
        if self.tunnel_id.is_none() {
            return Err(BindError::NotBound(self.id.to_string()));
        }

        self.tunnel_id = None;
        self.bind_status = BindStatus::Unbound;
        self.resolve_status = ResolveStatus::Unknown;
        Ok(())
    }

    pub fn enable(&mut self) {
        if self.status != DomainStatus::Deleted {
            self.status = DomainStatus::Active;
        }
    }

    pub fn disable(&mut self) {
        if self.status != DomainStatus::Deleted {
            self.status = DomainStatus::Disabled;
        }
    }

    pub fn mark_deleted(&mut self) {
        self.status = DomainStatus::Deleted;
    }

    pub fn set_verify_status(&mut self, verify_status: VerifyStatus) {
        self.verify_status = verify_status;
    }

    pub fn set_resolve_status(&mut self, resolve_status: ResolveStatus) {
        self.resolve_status = resolve_status;
    }

    pub fn set_dns_status(&mut self, dns_status: DnsStatus) {
        self.dns_status = dns_status;
    }
}

#[derive(Clone, Debug)]
pub struct DomainBuilder {
    id: DomainId,
    host: Host,
    aliases: Vec<Alias>,
    tunnel_id: Option<TunnelId>,
    record_type: RecordType,
    verify_status: VerifyStatus,
    resolve_status: ResolveStatus,
    dns_status: DnsStatus,
    status: DomainStatus,
}

impl DomainBuilder {
    pub fn new(id: DomainId, host: Host) -> Self {
        Self {
            id,
            host,
            aliases: Vec::new(),
            tunnel_id: None,
            record_type: RecordType::A,
            verify_status: VerifyStatus::Unverified,
            resolve_status: ResolveStatus::Unknown,
            dns_status: DnsStatus::NotChecked,
            status: DomainStatus::Pending,
        }
    }

    pub fn aliases(mut self, aliases: Vec<Alias>) -> Self {
        self.aliases = aliases;
        self
    }

    pub fn tunnel_id(mut self, tunnel_id: TunnelId) -> Self {
        self.tunnel_id = Some(tunnel_id);
        self
    }

    pub fn record_type(mut self, record_type: RecordType) -> Self {
        self.record_type = record_type;
        self
    }

    pub fn verify_status(mut self, verify_status: VerifyStatus) -> Self {
        self.verify_status = verify_status;
        self
    }

    pub fn resolve_status(mut self, resolve_status: ResolveStatus) -> Self {
        self.resolve_status = resolve_status;
        self
    }

    pub fn dns_status(mut self, dns_status: DnsStatus) -> Self {
        self.dns_status = dns_status;
        self
    }

    pub fn status(mut self, status: DomainStatus) -> Self {
        self.status = status;
        self
    }

    pub fn build(self) -> Result<Domain, DomainError> {
        ensure_unique_aliases(&self.host, &self.aliases)?;
        let bind_status = if self.tunnel_id.is_some() {
            BindStatus::Bound
        } else {
            BindStatus::Unbound
        };

        Ok(Domain {
            id: self.id,
            host: self.host,
            aliases: self.aliases,
            tunnel_id: self.tunnel_id,
            record_type: self.record_type,
            verify_status: self.verify_status,
            bind_status,
            resolve_status: self.resolve_status,
            dns_status: self.dns_status,
            status: self.status,
        })
    }
}

fn ensure_unique_aliases(host: &Host, aliases: &[Alias]) -> Result<(), DomainError> {
    let mut seen = BTreeSet::new();
    seen.insert(host.as_str().to_string());

    for alias in aliases {
        if !seen.insert(alias.as_str().to_string()) {
            return Err(DomainError::AlreadyExists(alias.as_str().to_string()));
        }
    }
    Ok(())
}
