# Domain Management Infrastructure

This crate provides Gate domain validation, repository, resolver, and service
primitives. In the release build, managed domain records are persisted through
the SQLite-backed repository when the default `sqlite` feature is enabled.

## Modules

- `model`: DomainId, TunnelId, Domain, Host, Alias, RecordType, and status models.
- `repository`: DomainRepository trait and SQLite repository implementation.
- `service`: DomainService use cases for create, delete, update, bind, unbind, enable, disable, rename, list, and search.
- `resolver`: HostResolver, RepositoryHostResolver, DnsResolver, and DnsChecker.
- `validator`: DomainValidator trait and RFC-oriented validator.
- `storage`: DomainStorage trait plus SQLite storage. In-memory storage is compiled only for tests.
- `error`: unified error hierarchy and DomainFailure trait.
- `config`: DomainConfig, ValidationMode, and storage kind configuration.
- `event`: domain event values without EventBus integration.
- `tests`: unit coverage for model, repository, resolver, service, storage, and validator behavior.

## Integration Rule

Runtime adapters should keep this crate as a domain boundary:

- HTTP and HTTPS routing resolve hosts through `HostResolver::resolve_host(host)`.
- TLS and ACME flows consume validated domain records through explicit runtime adapters.
- DNS checking is performed through `DnsResolver`; no provider credential or token is hardcoded here.
