---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-16
updated: 2026-03-17
version: 1.0
---

# Phase 2 Package And Registry Plan

## Objective

Land the smallest viable package and registry baseline for `blocks`:

- package manifest
- deterministic lockfile
- provider precedence
- file registry publish/fetch
- migration bridge for legacy roots
- `blocks pkg` JSON command surface

## Slices

1. `blocks-package` domain types and validation
2. `blocks pkg init|resolve|fetch|publish`
3. deterministic `blocks.lock`
4. provider precedence and file-registry baseline
5. bridge behavior for roots without `package.yaml`

## Acceptance

- RED tests in `crates/blocks-cli/tests/pkg_phase2_red.rs` pass
- `pkg fetch` does not expose checksum mismatch semantics
- `pkg resolve --compat` warns on unknown manifest keys while strict validation still rejects them
- `blocks.lock` stores concrete dependency versions and provider-selected sources
- bridge mode emits explicit warning text
- `conformance run package` verifies deterministic resolve + lockfile behavior through the public CLI surface
- a third-party package consumer can resolve an external dependency from a file registry without mirroring this repository layout
- docs and indexes reflect the landed Phase 2 baseline
