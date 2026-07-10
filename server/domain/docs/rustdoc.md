# Rust Doc Entry Points

The root Rust documentation lives in `server/domain/mod.rs`.

Important public entry points:

- `model::Domain`
- `model::DomainId`
- `model::TunnelId`
- `model::Host`
- `repository::DomainRepository`
- `repository::SqliteRepository`
- `service::DomainService`
- `validator::DomainValidator`
- `validator::RfcDomainValidator`
- `resolver::HostResolver`
- `resolver::DnsResolver`
- `resolver::DnsChecker`
- `storage::DomainStorage`
- `storage::SqliteDomainStorage`
- `error::DomainFailure`
