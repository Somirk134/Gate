# Communication Internals

Gate communication code coordinates client/server messaging, retries, dispatching, and transport integration.

## Maintainer boundaries

- Protocol compatibility is a release-critical boundary.
- Transport changes must be tested with realistic connection scenarios.
- Retry and reconnect behavior should be observable through logs and diagnostics.

## Release cleanup note

v0.9 release cleanup only reorganizes documentation and build configuration. It does not change communication behavior or wire format.
