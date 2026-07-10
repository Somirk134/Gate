# Gate Server TLS Infrastructure

This document describes the current TLS and certificate infrastructure used by
the server release candidate.

## Certificate Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Requested
    Requested --> PendingValidation
    PendingValidation --> Issued
    Issued --> Stored
    Stored --> Active
    Active --> ExpiringSoon: within 30 days
    ExpiringSoon --> RenewQueued
    RenewQueued --> Active: ACME renewal completed
    Active --> Expired
    Active --> Revoked
    Active --> Deleted
    Expired --> RenewQueued
    Revoked --> [*]
    Deleted --> [*]
```

## ACME Lifecycle

```mermaid
stateDiagram-v2
    [*] --> New
    New --> AccountReady: account created
    AccountReady --> OrderCreated: order started
    OrderCreated --> ChallengePrepared: challenge material ready
    ChallengePrepared --> ChallengeValidated: challenge accepted
    ChallengeValidated --> Finalized: CSR finalized
    Finalized --> CertificateIssued: certificate downloaded
    New --> Failed
    AccountReady --> Failed
    OrderCreated --> Failed
    ChallengePrepared --> Failed
    ChallengeValidated --> Failed
    Finalized --> Failed
```

## Renew Lifecycle

```mermaid
flowchart TD
    A[Daily scheduler tick] --> B[List certificate records]
    B --> C{Expires within 30 days?}
    C -->|No| D[Skip]
    C -->|Yes| E[Create renew decision]
    E --> F[Return renewal decision]
    F --> G[ACME executor]
```

## Module Relationship

```mermaid
flowchart LR
    Tls[TLS Provider Trait] --> Store[Certificate Store Trait]
    Tls --> Validator[Certificate Validator]
    Manager[CertificateManager Trait] --> Acme[AcmeProvider Trait]
    Manager --> Store
    Acme --> State[ACME State Machine]
    Store --> File[FileStore]
    Renew[Renew Scheduler] --> Store
    Renew --> Manager
    Crypto[Crypto Fingerprint] --> Parser[Certificate Parser]
    Parser --> Model[Certificate Model]
    Validator --> Model
```
