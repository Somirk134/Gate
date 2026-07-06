#[path = "../mod.rs"]
mod domain;

pub use domain::{config, error, event, model, repository, resolver, service, storage, traits, validator};

use domain::config::DomainConfig;
use domain::model::{DomainId, RecordType};
use domain::repository::MemoryRepository;
use domain::service::{CreateDomainRequest, DomainService};
use domain::validator::RfcDomainValidator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = DomainService::builder()
        .repository(MemoryRepository::new())
        .validator(RfcDomainValidator::new())
        .config(DomainConfig::default())
        .build()?;

    let output = service.create_domain(CreateDomainRequest {
        id: DomainId::new("domain-1")?,
        host: "api.gate.dev".to_string(),
        aliases: Vec::new(),
        record_type: RecordType::A,
    })?;

    println!("created domain: {}", output.value.host());
    Ok(())
}
