#[path = "../mod.rs"]
mod domain;

pub use domain::{config, error, event, model, repository, resolver, service, storage, traits, validator};

use domain::model::{Domain, DomainId, Host};
use domain::repository::{DomainRepository, MemoryRepository};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let domain = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?)
        .build()?;

    repository.create(domain)?;
    println!("domain count: {}", repository.list()?.len());
    Ok(())
}
