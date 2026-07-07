use crate::model::{Alias, Domain, DomainId, Host, TunnelId};
use crate::repository::{DomainRepository, MemoryRepository};

#[test]
fn memory_repository_supports_crud_and_lookup() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let id = DomainId::new("domain-1")?;
    let tunnel_id = TunnelId::new("tunnel-1")?;
    let domain = Domain::builder(id.clone(), Host::new("api.gate.dev")?).build()?;

    repository.create(domain.clone())?;
    assert_eq!(repository.find_by_id(&id)?, Some(domain.clone()));
    assert_eq!(
        repository.find_by_host(domain.host())?,
        Some(domain.clone())
    );
    assert!(repository.exists(domain.host())?);

    let bound = repository.bind_tunnel(&id, tunnel_id.clone())?;
    assert_eq!(bound.tunnel_id(), Some(&tunnel_id));
    assert_eq!(repository.find_by_tunnel(&tunnel_id)?.len(), 1);

    let unbound = repository.unbind_tunnel(&id)?;
    assert_eq!(unbound.tunnel_id(), None);

    let deleted = repository.delete(&id)?;
    assert_eq!(deleted.id(), &id);
    assert_eq!(repository.list()?.len(), 0);
    Ok(())
}

#[test]
fn memory_repository_rejects_duplicate_host() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let first = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?).build()?;
    let second = Domain::builder(DomainId::new("domain-2")?, Host::new("api.gate.dev")?).build()?;

    repository.create(first)?;
    let result = repository.create(second);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn memory_repository_rejects_alias_conflict() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let first = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?)
        .aliases(vec![Alias::new("www.gate.dev")?])
        .build()?;
    let second = Domain::builder(DomainId::new("domain-2")?, Host::new("edge.gate.dev")?)
        .aliases(vec![Alias::new("www.gate.dev")?])
        .build()?;

    repository.create(first)?;
    let result = repository.create(second);
    assert!(result.is_err());
    Ok(())
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_repository_persists_crud_and_recovers_after_reopen(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::model::{DnsStatus, DomainStatus, RecordType, ResolveStatus, VerifyStatus};
    use crate::repository::SqliteRepository;

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("domains.sqlite3");
    let repository = SqliteRepository::open(&db_path)?;

    let id = DomainId::new("domain-1")?;
    let tunnel_id = TunnelId::new("tunnel-1")?;
    let domain = Domain::builder(id.clone(), Host::new("api.gate.dev")?)
        .aliases(vec![Alias::new("www.gate.dev")?])
        .tunnel_id(tunnel_id.clone())
        .record_type(RecordType::Cname)
        .verify_status(VerifyStatus::Verified)
        .resolve_status(ResolveStatus::Resolved)
        .dns_status(DnsStatus::Matched)
        .status(DomainStatus::Active)
        .build()?;

    repository.create(domain.clone())?;

    let reopened = SqliteRepository::open(&db_path)?;
    assert_eq!(reopened.find_by_id(&id)?, Some(domain.clone()));
    assert_eq!(
        reopened.find_by_host(&Host::new("www.gate.dev")?)?,
        Some(domain.clone())
    );
    assert_eq!(reopened.find_by_tunnel(&tunnel_id)?.len(), 1);

    let mut updated = domain.clone();
    updated.disable();
    reopened.update(updated.clone())?;
    assert_eq!(
        SqliteRepository::open(&db_path)?.find_by_id(&id)?,
        Some(updated)
    );

    reopened.delete(&id)?;
    assert_eq!(SqliteRepository::open(&db_path)?.list()?.len(), 0);
    Ok(())
}
