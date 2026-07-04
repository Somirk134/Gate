# Domain Documentation

## Purpose

Domain documentation captures the business domain model and domain-driven
design artifacts for Gate.

## Contents

- **Glossary.md** — Ubiquitous language definitions
- **DomainModel.md** — Entity, aggregate, and value object descriptions
- **BoundedContexts.md** — Context mapping and boundaries
- **Events.md** — Domain events catalog
- **Rules.md** — Business rules and invariants

## Key Domain Concepts

| Concept | Description |
|---------|-------------|
| Client | A device running Gate client software |
| Tunnel | A persistent data path between client and server |
| Connection | An active TCP/UDP session through a tunnel |
| Proxy | Server-side endpoint that accepts external traffic |

## Why This Design

Domain isolation ensures that business logic remains independent of
infrastructure concerns. This makes the system testable and maintainable.

## Extension

Add new domain concepts as the product evolves. Update the ubiquitous
language glossary as terms are refined.
