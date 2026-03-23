---
status: active
owner: Developer
created: 2026-03-16
updated: 2026-03-16
version: 1.0
---

# ADR 004: Phase 2 Package And Registry Boundary

## Context

The repository needs a package baseline before runtime-platform and richer BCL work can proceed. The Phase 2 design must stay migration-safe and avoid pulling checksum/integrity or remote protocol complexity into the first implementation wave.

## Decision

Phase 2 adopts the following boundaries:

- `package.yaml` is the package-level source of truth for identity and dependencies
- descriptor semantics remain owned by `block.yaml`, `moc.yaml`, and `moc.bcl`
- `blocks.lock` is deterministic but does not carry checksum/signature guarantees yet
- registry access is provider-driven with workspace + file providers and a remote-ready seam only
- legacy roots without `package.yaml` use a migration bridge and are not publishable

## Consequences

- package resolution becomes explicit and testable
- current local descriptor workflows remain usable during migration
- checksum and remote protocol standardization remain deferred to later phases
