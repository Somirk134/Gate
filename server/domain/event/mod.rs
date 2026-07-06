use crate::model::{DnsStatus, Domain, DomainId, Host, TunnelId};

/// Domain event payloads are plain values only.
///
/// They are not connected to any EventBus in this phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainEvent {
    DomainCreated {
        domain_id: DomainId,
        host: Host,
    },
    DomainDeleted {
        domain_id: DomainId,
        host: Host,
    },
    DomainUpdated {
        domain_id: DomainId,
        host: Host,
    },
    DomainBound {
        domain_id: DomainId,
        tunnel_id: TunnelId,
    },
    DomainUnbound {
        domain_id: DomainId,
    },
    DomainEnabled {
        domain_id: DomainId,
    },
    DomainDisabled {
        domain_id: DomainId,
    },
    DnsChecked {
        domain_id: DomainId,
        host: Host,
        status: DnsStatus,
    },
}

impl DomainEvent {
    pub fn created(domain: &Domain) -> Self {
        Self::DomainCreated {
            domain_id: domain.id().clone(),
            host: domain.host().clone(),
        }
    }

    pub fn deleted(domain: &Domain) -> Self {
        Self::DomainDeleted {
            domain_id: domain.id().clone(),
            host: domain.host().clone(),
        }
    }

    pub fn updated(domain: &Domain) -> Self {
        Self::DomainUpdated {
            domain_id: domain.id().clone(),
            host: domain.host().clone(),
        }
    }
}
