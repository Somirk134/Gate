#[path = "../mod.rs"]
mod domain;

pub use domain::{config, error, event, model, repository, resolver, service, storage, traits, validator};

use domain::config::DomainConfig;
use domain::model::{DomainId, RecordType, TunnelId};
use domain::repository::MemoryRepository;
use domain::service::{CreateDomainRequest, DomainService};
use domain::validator::RfcDomainValidator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = DomainService::builder()
        .repository(MemoryRepository::new())
        .validator(RfcDomainValidator::new())
        .config(DomainConfig::default())
        .build()?;

    let domain_id = DomainId::new("domain-1")?;
    service.create_domain(CreateDomainRequest {
        id: domain_id.clone(),
        host: "api.gate.dev".to_string(),
        aliases: Vec::new(),
        record_type: RecordType::A,
    })?;

    let output = service.bind_tunnel(&domain_id, TunnelId::new("tunnel-1")?)?;
    println!("bound domain: {}", output.value.host());
    Ok(())
}
