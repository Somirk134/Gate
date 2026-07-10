# Runtime Internals

This page is a high-level internal reference for maintainers. It is not a user guide.

## Scope

Runtime code owns local tunnel lifecycle, connection forwarding, statistics collection, and integration with server gateway behavior. The v0.9 release cleanup does not modify runtime behavior.

## Boundaries

- TCP, HTTP, and HTTPS data-plane behavior belongs to the runtime and gateway code paths.
- Release engineering changes should not alter packet handling, forwarding semantics, or session behavior.
- Documentation and CI changes must remain separate from runtime refactors.

## Maintainer checklist

When touching runtime code:

1. Document the data-plane impact.
2. Add focused tests for the protocol or forwarding behavior being changed.
3. Run workspace tests and integration tests.
4. Avoid mixing runtime changes with release packaging changes.
