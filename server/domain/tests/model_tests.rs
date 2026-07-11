use crate::error::{BindError, DomainError};
use crate::model::{Alias, BindStatus, Domain, DomainId, DomainStatus, Host, RecordType, TunnelId};

#[test]
fn host_normalizes_case_and_root_dot() -> Result<(), Box<dyn std::error::Error>> {
    let host = Host::new("Api.Gate.Dev.")?;
    assert_eq!(host.as_str(), "api.gate.dev");
    Ok(())
}

#[test]
fn domain_builder_sets_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let domain = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?)
        .record_type(RecordType::Aaaa)
        .build()?;

    assert_eq!(domain.record_type(), &RecordType::Aaaa);
    assert_eq!(domain.bind_status(), &BindStatus::Unbound);
    assert_eq!(domain.status(), &DomainStatus::Pending);
    Ok(())
}

#[test]
fn duplicate_alias_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let result = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?)
        .aliases(vec![
            Alias::new("www.gate.dev")?,
            Alias::new("WWW.GATE.DEV")?,
        ])
        .build();

    assert!(matches!(result, Err(DomainError::AlreadyExists(_))));
    Ok(())
}

#[test]
fn alias_matching_primary_host_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let result = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?)
        .aliases(vec![Alias::new("api.gate.dev")?])
        .build();

    assert!(matches!(result, Err(DomainError::AlreadyExists(_))));
    Ok(())
}

#[test]
fn domain_bind_and_unbind_updates_state() -> Result<(), Box<dyn std::error::Error>> {
    let mut domain =
        Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?).build()?;
    let tunnel_id = TunnelId::new("tunnel-1")?;

    domain.bind(tunnel_id.clone())?;
    assert_eq!(domain.tunnel_id(), Some(&tunnel_id));
    assert_eq!(domain.bind_status(), &BindStatus::Bound);

    domain.unbind()?;
    assert_eq!(domain.tunnel_id(), None);
    assert_eq!(domain.bind_status(), &BindStatus::Unbound);
    Ok(())
}

#[test]
fn domain_rebind_replaces_existing_tunnel() -> Result<(), Box<dyn std::error::Error>> {
    let mut domain =
        Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?).build()?;
    let first = TunnelId::new("tunnel-1")?;
    let second = TunnelId::new("tunnel-2")?;

    domain.bind(first)?;
    domain.rebind(second.clone())?;
    assert_eq!(domain.tunnel_id(), Some(&second));
    assert_eq!(domain.bind_status(), &BindStatus::Bound);
    Ok(())
}

#[test]
fn binding_disabled_domain_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let mut domain =
        Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?).build()?;
    domain.disable();

    let result = domain.bind(TunnelId::new("tunnel-1")?);
    assert!(matches!(result, Err(BindError::DisabledDomain(_))));
    Ok(())
}
