# Architecture Decision Records (ADRs)

## Purpose

Architecture Decision Records capture important architectural decisions
made during the project lifecycle, including context, options considered,
and rationale.

## What is an ADR?

An ADR is a short document (1-2 pages) that records:

- **Context** — What is the problem or situation?
- **Options** — What alternatives were considered?
- **Decision** — What was chosen and why?
- **Consequences** — What are the trade-offs and implications?

## ADR Template

```markdown
# ADR-NNN: Title

## Status

[Proposed | Accepted | Deprecated | Superseded]

## Context

[Description of the problem and context]

## Options

1. Option A
2. Option B

## Decision

[Chosen option and rationale]

## Consequences

[Positive and negative consequences]
```

## ADR Index

| ADR | Title | Status |
|-----|-------|--------|
| 001 | Monorepo structure with Cargo workspace | Accepted |

## Why This Design

ADRs provide a permanent record of why decisions were made, preventing
repeated discussions and helping new team members understand the project's
evolution.

## Extension

Create a new ADR for each significant architectural decision. Number
sequentially. Update status when decisions are revisited.
