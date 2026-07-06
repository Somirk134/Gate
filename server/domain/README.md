# Domain Management Infrastructure

This directory is a zero-intrusion Domain Management Infrastructure for Gate.
It is not wired into the current Cargo workspace, HTTP runtime, HTTPS runtime,
TLS infrastructure, tunnel runtime, communication layer, dashboard, or existing
configuration files.

## Modules

- `model`: DomainId, TunnelId, Domain, Host, Alias, RecordType, and status models.
- `repository`: DomainRepository trait and MemoryRepository.
- `service`: DomainService use cases for create, delete, update, bind, unbind, enable, disable, rename, list, and search.
- `resolver`: HostResolver, RepositoryHostResolver, DnsResolver, DnsChecker, and MockDnsResolver.
- `validator`: DomainValidator trait and RFC-oriented validator.
- `storage`: DomainStorage trait and memory storage; SQLite, Redis, JSON, and file storage are reserved as traits.
- `error`: unified error hierarchy and DomainFailure trait.
- `traits`: future injection ports for HTTPS, TLS, Certificate, ACME, and DNS providers.
- `config`: DomainConfig, ValidationMode, and storage kind configuration.
- `event`: domain event values without EventBus integration.
- `tests`: standalone unit tests compiled through `rustc --test server/domain/mod.rs`.
- `examples`: standalone usage examples.

## Standalone Test

```powershell
rustc --test server\domain\mod.rs -o target\domain_mod_tests.exe
.\target\domain_mod_tests.exe
```

## Integration Rule

Future runtimes should depend on traits only:

- HTTP runtime calls `HostResolver::resolve_host(host)` and receives `TunnelId`.
- HTTPS runtime calls the same resolver, then its own TLS layer independently.
- TLS/ACME infrastructure receives domain events or service outputs and calls reserved ports.
- DNS providers implement `DnsProviderPort` or `DnsResolver`; no provider is hardcoded here.
