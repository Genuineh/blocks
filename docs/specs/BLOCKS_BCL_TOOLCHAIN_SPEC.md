---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-13
updated: 2026-03-16
version: 1.0
related_prds:
  - docs/prds/BLOCKS_BCL_TOOLCHAIN_PLAN.md
---

# Blocks Platform And BCL Specification

## Overview

This specification defines the target technical boundary for the repository as the package, runtime, and language platform for `blocks`.

The repository remains a standards and platform layer. It does not become a product-management or deploy-control system. But it now explicitly includes:

- package identity, discovery, resolution, and lock semantics
- a Rust-native runtime platform with stable host contracts above concrete Rust runtimes
- `BCL` as a first-class language/compiler surface rather than only a constrained `moc` assist path

## Goals

- provide coherent public CLI surfaces for package, runtime, and BCL workflows
- make deterministic resolution, formatting, compiler output, and diagnostics first-class platform capabilities
- provide a reusable runtime host contract that can be implemented on top of multiple Rust runtimes
- provide conformance, compatibility, and migration paths for third-party adopters
- preserve a migration bridge from current `moc.yaml`-centric workflows while shifting long-term authority toward package-aware BCL compilation

## Non-Goals

- deploy orchestration or product-management workflow tooling
- forcing a single runtime host implementation on all adopters
- replacing Rust as the core implementation language of the platform
- editor/IDE protocol work in the first architectural realignment wave
- pretending the current local runner catalog is the final package/runtime architecture

## Architecture

### Components

- `crates/blocks-cli`
  - primary public command surface for package, runtime, language, conformance, troubleshooting, and migration workflows
- `crates/blocks-contract`
  - canonical contract parser, validator, normalization source, and compatibility input for block-facing contracts
- `crates/blocks-package` (planned)
  - package manifest model, package identity, version constraints, lockfile structures, and resolver input types
- `crates/blocks-registry`
  - local and remote catalog/index access, package metadata discovery, and publish/fetch integration points
- `crates/blocks-moc`
  - legacy/current `moc.yaml` validator, migration bridge, and descriptor compatibility checker
- `crates/blocks-bcl`
  - canonical BCL parser, formatter, semantic checker, compiler pipeline, and explainability source
- `crates/blocks-runtime`
  - runtime contracts, execution envelopes, diagnostics contracts, artifact boundaries, and host-facing APIs
- `crates/blocks-runtime-host-*` (planned)
  - concrete host implementations above different Rust runtimes or host environments
- checked-in template and fixture assets
  - deterministic scaffold sources, package fixtures, conformance cases, and migration baselines
- repository gate scripts
  - stable command entrypoints for repository-level verification and external CI reuse

### Boundary Rules

- package resolution is a platform concern, not an ad hoc CLI convenience feature
- runtime host compatibility must be expressed through stable Rust traits/data contracts, not through duplicated glue per command
- BCL compilation must target platform artifacts through explicit lowering phases rather than hidden side effects
- legacy `moc.yaml` validation remains supported during migration, but new design work should not treat descriptor parity as the final architecture
- all automation-oriented new commands must offer a stable `--json` mode
- no first-wave requirement exists for IDE/LSP/editor-specific protocol support

### Namespace Rules

- new command families should follow resource-first namespaces:
  - `blocks block ...`
  - `blocks moc ...`
  - `blocks pkg ...`
  - `blocks runtime ...`
  - `blocks bcl ...`
  - `blocks conformance ...`
  - `blocks catalog ...`
  - `blocks compat ...`
  - `blocks upgrade ...`
- legacy surfaces such as `list/show/run/search blocks` and `blocks moc bcl ...` may remain as compatibility aliases during migration

### Data Flow

1. package commands create, publish, fetch, resolve, and lock package graphs
2. format commands canonicalize package manifests, descriptors, and BCL source before review or build
3. check commands validate package metadata, contract metadata, and language sources
4. build/compile commands lower BCL packages into runtime-consumable artifacts and optional compatibility descriptors
5. runtime commands materialize execution plans through the stable runtime host boundary
6. test/eval/conformance commands execute evidence assets and report deterministic pass/fail summaries
7. doctor/graph/explain commands consume resolver, compiler, and runtime outputs to produce repair-oriented summaries
8. compat/upgrade commands compare and migrate package/compiler/runtime artifacts across version drift

## CLI Specification

### `blocks pkg init`

- **Input**: target directory, package kind (`block|moc|bcl`), package id, optional language/runtime profile
- **Output**: scaffolded package plus `ScaffoldReport`
- **Errors**: invalid package id, path conflict, unsupported package kind/profile combination

### `blocks pkg resolve`

- **Input**: package root or manifest path, optional registry configuration, optional `--compat`, optional `--lock`, optional `--json`
- **Output**: `ResolveReport` with selected versions, sources, dependency graph, and lockfile preview/write result
- **Errors**: unsatisfied constraints, registry access failure, conflicting release, conflicting lock state

### `blocks pkg publish`

- **Input**: package root, optional target registry, optional `--json`
- **Output**: `PublishReport`
- **Errors**: invalid package metadata, unpublished dependencies, registry rejection

### `blocks pkg fetch`

- **Input**: package id or lockfile, optional registry configuration, optional `--json`
- **Output**: `FetchReport`
- **Errors**: unresolved package, source unavailable, unsupported source, cache write failure

### `blocks block init`

- **Input**: target directory or `block-id`, implementation kind/target options, optional example/evaluator scaffolds
- **Output**: scaffolded block package plus `ScaffoldReport`
- **Errors**: invalid identifier, existing target path conflict, unsupported kind/target combination

### `blocks block fmt`

- **Input**: `block.yaml` path or block root path
- **Output**: canonical formatted descriptor; optional in-place write mode
- **Errors**: invalid YAML, unsupported descriptor shape

### `blocks block check`

- **Input**: `block.yaml` path or block root path, optional `--json`
- **Output**: `CheckReport` with errors/warnings/normalized metadata summary
- **Errors**: parse failure, invalid contract definition

### `blocks block test`

- **Input**: block root path or block ID, optional fixture selector
- **Output**: executable evidence summary for block-local tests/examples
- **Errors**: missing evidence assets, failed test command, unsupported target

### `blocks block eval`

- **Input**: block root path or block ID, optional fixture/example selector
- **Output**: evaluation summary over block-local evaluators/fixtures
- **Errors**: missing evaluator asset, invalid evaluation output, failed evaluator command

### `blocks block doctor`

- **Input**: block root path or block ID, optional `--json`
- **Output**: repair-oriented summary across contract issues, missing executable evidence, and latest diagnostics when present
- **Errors**: unresolved block, unreadable artifacts

### `blocks moc init`

- **Input**: moc ID, type, language, backend mode when relevant
- **Output**: scaffolded moc package plus `ScaffoldReport`
- **Errors**: invalid moc type, missing required backend mode, path conflict

### `blocks moc fmt`

- **Input**: `moc.yaml` path
- **Output**: canonical formatted descriptor; optional in-place write mode
- **Errors**: invalid YAML, unsupported descriptor shape

### `blocks moc check`

- **Input**: blocks root + `moc.yaml` path, optional `--json`
- **Output**: `CheckReport` backed by `blocks-moc` validation and protocol checks
- **Errors**: manifest parse failure, invalid descriptor, protocol mismatch

### `blocks moc doctor`

- **Input**: blocks root + `moc.yaml` path, optional `--json`
- **Output**: repair-oriented summary across descriptor validity, runtime launcher availability, latest diagnostic trace, and protocol health
- **Errors**: unreadable manifest, unresolved diagnostics, malformed artifacts

### `blocks runtime check`

- **Input**: block root or `block.yaml`, optional host profile filter, optional `--json`
- **Output**: `RuntimeCheckReport` covering host compatibility, runtime capabilities, and diagnostics contract support
- **Errors**: unsupported host profile, missing runtime hooks, incompatible contract/host pairing

### `blocks runtime run`

- **Input**: compiled artifact or package root, optional host profile, optional `--json`
- **Output**: runtime execution summary or stream handle metadata
- **Errors**: unresolved artifact, host startup failure, execution contract failure

### `blocks bcl init`

- **Input**: target package or existing `moc.yaml` path, optional starter mode
- **Output**: starter BCL source plus `ScaffoldReport`
- **Errors**: unsupported descriptor, conflicting output path

### `blocks bcl fmt`

- **Input**: BCL source path
- **Output**: canonical formatted BCL source; optional in-place write mode
- **Errors**: parse failure, unsupported syntax

### `blocks bcl check`

- **Input**: package root or BCL source path, optional `--json`
- **Output**: structured syntax/semantic/package-resolution report
- **Errors**: syntax, semantic, resolution, or compatibility failures

### `blocks bcl graph`

- **Input**: package root or BCL source path, optional `--json`
- **Output**: node/edge graph across modules, packages, blocks, protocols, flows, and lowered runtime units
- **Errors**: parse failure, graph construction blocked by invalid semantics

### `blocks bcl explain`

- **Input**: package root or BCL source path, optional `--json`
- **Output**: repair-oriented explanation layered on top of check/build failures; on success, concise compiler summary
- **Errors**: invalid source, missing assets, unresolved packages

### `blocks bcl build`

- **Input**: package root or BCL source path, optional target profile, optional `--lock`, optional `--json`
- **Output**: `BuildReport` with lowered artifacts, selected package graph, and compatibility outputs
- **Errors**: resolution failure, semantic failure, artifact emission failure

Compatibility aliases:

- `blocks moc bcl init`
- `blocks moc bcl fmt`
- `blocks moc bcl validate`
- `blocks moc bcl plan`
- `blocks moc bcl emit`
- `blocks moc bcl graph`
- `blocks moc bcl explain`

These may remain during migration, but new documentation should prefer the top-level `blocks bcl ...` namespace.

### `blocks conformance run`

- **Input**: suite type (`block`, `moc`, `bcl`, `package`, `runtime`), repository path or target path, optional fixture subset, optional provider config for package/runtime suites, optional `--json`
- **Output**: deterministic `ConformanceReport`
- **Errors**: missing normative assets, failed suite, unsupported target layout

Package-suite baseline:

- `blocks conformance run package <package-root|package.yaml> [--provider ...] [--compat] [--json]`
- runs `pkg resolve`, `pkg resolve --lock`, and a repeated lock-writing pass
- fails when resolution is unsatisfied, `blocks.lock` is not written, or repeated runs change JSON/lockfile bytes
- is the minimum public verification surface for third-party repositories that only adopt package resolution

Runtime-suite baseline:

- `blocks conformance run runtime <block-root|block.yaml> [--host ...] [--input <json-file>] [--json]`
- runs host compatibility checks, executes the same block contract on the selected runtime host profiles, and verifies output parity across successful hosts
- defaults to the Phase 3 baseline host profiles: `sync-cli` and `tokio-service`

BCL-suite baseline:

- `blocks conformance run bcl <package-root|package.yaml|moc-root|moc.bcl> [--provider ...] [--compat] [--check-against <moc.yaml>] [--gate-mode <off|warn|error>] [--json]`
- package-root mode runs `bcl check`, `bcl graph`, and `bcl build`, and may compare the emitted compatibility artifact against `moc.yaml`
- legacy workspace-layout mode still accepts `blocks conformance run bcl <blocks-root> <moc-root|moc.bcl> ...` and continues to run the migration-era `moc bcl check|plan|emit` parity path

### `blocks catalog export`

- **Input**: local cache, registry config, or repository root, optional filters, optional `--json`
- **Output**: stable catalog entries for packages and exported block capabilities
- **Errors**: registry scan failure, invalid metadata, inaccessible source

### `blocks catalog search`

- **Input**: registry config, query/filter options, optional `--json`
- **Output**: filtered `CatalogEntry[]` across local and remote metadata sources
- **Errors**: invalid filter expression, registry scan failure

### `blocks compat`

- **Input**: before/after descriptors, package manifests, or build outputs; target kind (`block`, `moc`, `bcl`, `package`, `runtime`)
- **Output**: `CompatReport` describing additive and breaking changes over schemas, protocols, package APIs, and runtime expectations
- **Errors**: unreadable inputs, incomparable targets

### `blocks upgrade`

- **Input**: target descriptor/package path plus optional rule set, optional `--write`
- **Output**: migrated descriptor, manifest, or source preview and `UpgradeReport`
- **Errors**: unsupported migration path, lossy migration without explicit override

## Data Models

### ScaffoldReport

| Field | Type | Description |
|-------|------|-------------|
| kind | string | `block` \| `moc` \| `bcl` \| `package` |
| id | string | scaffold target identifier |
| created_paths | array[string] | files/directories created |
| warnings | array[string] | non-fatal scaffold warnings |

### ResolveReport

| Field | Type | Description |
|-------|------|-------------|
| status | string | `ok` \| `error` |
| root_package | string | resolved root package id |
| selected_versions | array[object] | chosen package versions and sources |
| dependency_graph | object | normalized dependency graph |
| lockfile_path | string? | written or proposed lockfile path |
| errors | array[object] | structured resolution failures |

### CheckReport

| Field | Type | Description |
|-------|------|-------------|
| status | string | `ok` \| `warn` \| `error` |
| target_kind | string | `block` \| `moc` \| `bcl` \| `package` |
| target | string | path or identifier |
| errors | array[object] | structured fatal issues |
| warnings | array[object] | structured non-fatal issues |
| normalized_summary | object | stable normalized metadata summary |

### ConformanceReport

| Field | Type | Description |
|-------|------|-------------|
| suite | string | `block` \| `moc` \| `bcl` \| `package` \| `runtime` |
| status | string | `ok` \| `error` |
| cases_run | integer | number of cases executed |
| failures | array[object] | failed case summaries |
| warnings | array[string] | non-fatal gate warnings |
| artifacts | array[string] | emitted artifact paths |

### RuntimeCheckReport

| Field | Type | Description |
|-------|------|-------------|
| status | string | `ok` \| `warn` \| `error` |
| host_profile | string | runtime host profile under test |
| supported_capabilities | array[string] | host capabilities exposed |
| missing_capabilities | array[string] | required but absent capabilities |
| diagnostics_contract | object | supported diagnostics/artifact policy surface |

### CatalogEntry

| Field | Type | Description |
|-------|------|-------------|
| package_id | string | stable package identity |
| package_kind | string | `block` \| `moc` \| `bcl` |
| version | string | package version |
| source | object | registry/cache/source metadata |
| exported_capabilities | array[object] | exported block/runtime/language capabilities |
| evidence | object | evidence presence summary |

## Migration Rules

- `moc.yaml` remains supported during migration, but it is treated as a compatibility descriptor rather than the permanent center of gravity.
- current `blocks moc bcl ...` flows remain supported as aliases while top-level `blocks bcl ...` becomes the preferred namespace.
- the existing local runner catalog remains a bootstrap execution path until the runtime host boundary is proven on at least two host profiles.
- new feature work should prefer package-aware and runtime-host-aware APIs over adding more local-repository-only command semantics.

## Phase 2 Normative References

- package manifest and lockfile contract: `docs/specs/BLOCKS_PACKAGE_MODEL_SPEC.md`
- registry provider baseline and fetch taxonomy: `docs/specs/BLOCKS_REGISTRY_BASELINE_SPEC.md`

## Acceptance

- package identity, resolver inputs, and lockfile outputs are defined before registry features expand
- runtime host contracts are specified before more execution glue is added
- BCL is specified as a language/compiler surface rather than only a descriptor emitter
- migration aliases and compatibility rules are explicit, not implicit
