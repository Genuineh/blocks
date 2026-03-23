---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-16
updated: 2026-03-16
version: 1.0
---

# Blocks Package Model Specification

## Overview

This specification defines the Phase 2 package baseline for `blocks`.

Phase 2 introduces a package manifest and lockfile model without changing runtime authority. Descriptor semantics remain owned by `block.yaml`, `moc.yaml`, and `moc.bcl`.

## Package Manifest

Each package root may contain `package.yaml`.

Required fields:

- `api_version`: must be `blocks.pkg/v1`
- `kind`: `block|moc|bcl`
- `id`: lowercase dot-separated identifier
- `version`: semver-like `major.minor.patch`
- `descriptor.path`: relative descriptor path

Optional fields:

- `dependencies[]`
- `source`
- `metadata`

Kind rules:

- `block` packages must point to `block.yaml`
- `moc` packages must point to `moc.yaml`
- `bcl` packages must point to `moc.bcl`

Validation rules:

- duplicate dependencies by `(id, kind)` are rejected
- self-dependency is rejected
- unknown top-level keys are rejected in strict mode and warned in compatibility mode
- compatibility mode is entered through `blocks pkg resolve --compat`

## Lockfile

The Phase 2 lockfile is `blocks.lock`.

Required top-level fields:

- `version`
- `root`
- `providers`
- `resolved`

Determinism rules:

- `resolved[]` is sorted by `(id, kind, version, source.location)`
- nested `dependencies[]` use the same ordering
- dependency entries record concrete resolved versions, not requirement strings
- timestamps and other nondeterministic fields are not allowed

Deferred from Phase 2:

- checksum pinning
- signature verification
- remote integrity guarantees

## Authority Matrix

| Concern | Source of truth |
|---|---|
| package identity and dependency graph | `package.yaml` |
| block contract semantics | `block.yaml` |
| moc structure and protocol semantics | `moc.yaml` |
| BCL syntax and semantic rules | `moc.bcl` |

Migration bridge rules:

- if `package.yaml` exists, it is the package authority for that root
- bridge synthesis is fallback-only for local legacy roots without `package.yaml`
- bridge-derived packages are not publishable
