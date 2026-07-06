use crate::config::DomainConfig;
use crate::event::DomainEvent;
use crate::model::{DomainId, DomainStatus, RecordType, TunnelId};
use crate::repository::MemoryRepository;
use crate::service::{CreateDomainRequest, DomainService, SearchDomainRequest, UpdateDomainRequest};
use crate::validator::RfcDomainValidator;

#[test]
fn domain_service_create_bind_search_rename_disable_delete() -> Result<(), Box<dyn std::error::Error>>
{
    let service = DomainService::builder()
        .repository(MemoryRepository::new())
        .validator(RfcDomainValidator::new())
        .config(DomainConfig::builder().max_domains(10).build())
        .build()?;

    let id = DomainId::new("domain-1")?;
    let created = service.create_domain(CreateDomainRequest {
        id: id.clone(),
        host: "api.gate.dev".to_string(),
        aliases: vec!["www.gate.dev".to_string()],
        record_type: RecordType::A,
    })?;
    assert!(matches!(
        created.event,
        Some(DomainEvent::DomainCreated { .. })
    ));

    let tunnel_id = TunnelId::new("tunnel-1")?;
    let bound = service.bind_tunnel(&id, tunnel_id.clone())?;
    assert_eq!(bound.value.tunnel_id(), Some(&tunnel_id));

    let search = service.search(SearchDomainRequest {
        keyword: Some("api".to_string()),
        tunnel_id: Some(tunnel_id),
        status: None,
        enabled: Some(true),
    })?;
    assert_eq!(search.value.len(), 1);

    let renamed = service.rename(&id, "edge.gate.dev")?;
    assert_eq!(renamed.value.host().as_str(), "edge.gate.dev");

    let disabled = service.disable(&id)?;
    assert_eq!(disabled.value.status(), &DomainStatus::Disabled);

    let enabled = service.enable(&id)?;
    assert_eq!(enabled.value.status(), &DomainStatus::Active);

    service.update_domain(UpdateDomainRequest {
        id: id.clone(),
        host: None,
        aliases: Some(vec!["alt.gate.dev".to_string()]),
        record_type: Some(RecordType::Txt),
    })?;

    let deleted = service.delete_domain(&id)?;
    assert_eq!(deleted.value.id(), &id);
    assert_eq!(service.list()?.value.len(), 0);
    Ok(())
}

#[test]
fn domain_service_enforces_max_domains() -> Result<(), Box<dyn std::error::Error>> {
    let service = DomainService::builder()
        .repository(MemoryRepository::new())
        .validator(RfcDomainValidator::new())
        .config(DomainConfig::builder().max_domains(1).build())
        .build()?;

    service.create_domain(CreateDomainRequest {
        id: DomainId::new("domain-1")?,
        host: "api.gate.dev".to_string(),
        aliases: Vec::new(),
        record_type: RecordType::A,
    })?;

    let result = service.create_domain(CreateDomainRequest {
        id: DomainId::new("domain-2")?,
        host: "edge.gate.dev".to_string(),
        aliases: Vec::new(),
        record_type: RecordType::A,
    });
    assert!(result.is_err());
    Ok(())
}
