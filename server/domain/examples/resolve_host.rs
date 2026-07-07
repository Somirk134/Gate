#[path = "../mod.rs"]
mod domain;

pub use domain::{
    config, error, event, model, repository, resolver, service, storage, traits, validator,
};

use domain::model::{Domain, DomainId, Host, TunnelId};
use domain::repository::{DomainRepository, MemoryRepository};
use domain::resolver::{HostResolver, RepositoryHostResolver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let tunnel_id = TunnelId::new("tunnel-1")?;
    let domain = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?)
        .tunnel_id(tunnel_id.clone())
        .build()?;

    repository.create(domain)?;

    let resolver = RepositoryHostResolver::new(repository);
    let resolved = resolver.resolve_host("api.gate.dev")?;
    println!("resolved tunnel: {resolved}");
    Ok(())
}
