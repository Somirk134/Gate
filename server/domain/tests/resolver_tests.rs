use crate::config::DomainConfig;
use crate::model::{Domain, DomainId, Host, RecordType, TunnelId};
use crate::repository::{DomainRepository, MemoryRepository};
use crate::resolver::{
    DnsAnswer, DnsChecker, DnsQuery, HostResolver, MockDnsResolver, RepositoryHostResolver,
};

#[test]
fn host_resolver_maps_host_to_tunnel() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let id = DomainId::new("domain-1")?;
    let tunnel_id = TunnelId::new("tunnel-1")?;
    let domain = Domain::builder(id.clone(), Host::new("api.gate.dev")?)
        .tunnel_id(tunnel_id.clone())
        .build()?;
    repository.create(domain)?;

    let resolver = RepositoryHostResolver::new(repository);
    assert_eq!(resolver.resolve_host("API.GATE.DEV")?, tunnel_id);
    Ok(())
}

#[test]
fn host_resolver_rejects_unbound_host() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let domain = Domain::builder(DomainId::new("domain-1")?, Host::new("api.gate.dev")?)
        .build()?;
    repository.create(domain)?;

    let resolver = RepositoryHostResolver::new(repository);
    let result = resolver.resolve_host("api.gate.dev");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn mock_dns_checker_reports_match() -> Result<(), Box<dyn std::error::Error>> {
    let host = Host::new("api.gate.dev")?;
    let query = DnsQuery {
        host,
        record_type: RecordType::A,
    };
    let resolver = MockDnsResolver::new().with_record(
        query.clone(),
        DnsAnswer {
            record_type: RecordType::A,
            ttl: 60,
            values: vec!["203.0.113.10".to_string()],
        },
    )?;
    let config = DomainConfig::builder()
        .server_address("203.0.113.10")
        .build();
    let checker = DnsChecker::new(resolver, config);
    let result = checker.check(&query)?;

    assert!(result.resolved_to_server);
    assert_eq!(result.ttl, 60);
    Ok(())
}
