# Rust Doc Entry Points

The root Rust documentation lives in `server/domain/mod.rs`.

Important public entry points:

- `model::Domain`
- `model::DomainId`
- `model::TunnelId`
- `model::Host`
- `repository::DomainRepository`
- `repository::MemoryRepository`
- `service::DomainService`
- `validator::DomainValidator`
- `validator::RfcDomainValidator`
- `resolver::HostResolver`
- `resolver::DnsResolver`
- `resolver::DnsChecker`
- `storage::DomainStorage`
- `storage::MemoryDomainStorage`
- `traits::*Port`
- `error::DomainFailure`

Generate standalone docs after wiring into a crate with:

```powershell
rustdoc server\domain\mod.rs --crate-name gate_domain_management
```
