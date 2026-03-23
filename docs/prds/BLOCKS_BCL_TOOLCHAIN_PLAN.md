---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-13
updated: 2026-03-16
version: 1.0
---

# Blocks Platform And BCL Plan

## Summary

Define a unified roadmap that turns this repository into the standard package, runtime, and language platform for `blocks`.

The target state is no longer just a local authoring/checking toolchain:

- `blocks` must own a repository-aware package model for `block`, `moc`, and BCL assets
- the runtime layer must become a higher-level Rust-native platform that encapsulates runtime glue while remaining compatible with arbitrary Rust runtimes
- `BCL` must evolve from a constrained `moc` assist DSL into a first-class language with stable source, compiler, package resolution, and lowered runtime artifacts

The repository still does not aim to become a product-management or deployment system, but it now explicitly does aim to provide the core standards and implementation baseline for package discovery, dependency resolution, runtime hosting, and language compilation.

## Problem

The repository already provides useful pieces:

- block and moc specifications
- a local CLI and validation baseline
- a local Rust runner catalog
- a constrained BCL MVP
- proof-slice mocs across backend and frontend paths

But the system boundary is still too small for the stated long-term vision:

- discovery is still local-repository oriented rather than package-oriented
- dependency management has no remote registry, version solver, or lockfile model
- execution is still centered on a reference Rust dispatch path instead of a reusable runtime platform boundary
- BCL is still constrained to `validate -> plan -> emit moc.yaml`, which is useful but does not make it the actual source language of the system
- active plans and whitepapers still describe different futures, which invites architecture drift

Without a unified plan, the repository risks becoming a pile of partial tools instead of the actual platform layer that future `blocks` adoption depends on.

## Users

- maintainers of block libraries who need package publication, discovery, versioning, and conformance
- moc authors who need a stable runtime platform and resolved dependencies rather than only local descriptors
- BCL authors who need a real language/compiler workflow
- AI agents that need deterministic package metadata, stable compiler outputs, and machine-readable diagnostics
- platform maintainers who need compatibility gates across packages, runtimes, and language revisions

## Requirements

### Must Have

- keep repository scope explicit:
  - provide standards, package/discovery infrastructure, runtime platform, compiler/tooling, conformance, and reference assets
  - do not take ownership of product requirement intake or deploy orchestration
- establish a repository-aware and externally extensible package model for:
  - `block`
  - `moc`
  - BCL packages/modules
- define a package identity and dependency system with:
  - package manifests
  - version constraints
  - deterministic resolution
  - lockfiles
  - local cache and remote index support
- evolve the runtime into a higher-level Rust-native platform that:
  - encapsulates block invocation/runtime glue
  - exposes stable host traits and execution contracts
  - supports multiple Rust runtime implementations behind a common boundary
- promote `BCL` to a first-class language:
  - stable parser and formatter
  - semantic analysis
  - dependency resolution against package metadata
  - lowering/compilation into runtime-consumable artifacts
  - compatibility and migration rules across language revisions
- keep machine-readable output (`--json`) as a first-class contract for automation-oriented commands
- preserve deterministic formatting, canonicalization, and reproducible builds
- keep public repository guides as the source of truth instead of ad hoc internal-only instructions

### Should Have

- local + remote catalog discovery optimized for AI and scripted selection
- package publish/fetch/install flows that work in both local and external repositories
- runtime profiles for common Rust execution modes (`tokio`, `tauri`, single-process CLI/service hosts)
- `doctor`/`graph`/`explain` surfaces for package resolution, compiler output, and runtime health
- reusable conformance suites for third-party repositories and package publishers

### Nice to Have

- reverse generation or migration from existing `moc.yaml` to richer BCL source
- richer standard libraries and starter packs for common backend/frontend patterns
- optional non-Rust runtime adapters after the Rust-native platform boundary is stable

## Non-Goals

- becoming a general-purpose deployment control plane
- replacing Rust as the implementation language for the platform core
- forcing a single concrete Rust runtime implementation on all adopters
- making the first delivery wave depend on IDE/LSP/editor integrations
- curating every production block for every domain inside this repository

## User Stories

- As a package author, I want to publish and resolve `block` packages with version constraints so that reuse works across repositories instead of only inside one workspace.
- As a moc author, I want to target a stable Rust-native platform API rather than wiring raw runtime details by hand.
- As a runtime integrator, I want to implement the host boundary on top of my preferred Rust runtime without rewriting contract, tracing, and diagnostic semantics.
- As a BCL author, I want BCL to be the reviewed source artifact, with deterministic compiler output and dependency resolution.
- As an AI agent, I want stable package metadata, graph outputs, compiler diagnostics, and lockfiles so that end-to-end generation is reproducible.

## Success Metrics

- a third-party repository can resolve and lock external `block` packages deterministically
- at least two distinct Rust runtime hosts can execute the same runtime contract without changing block contracts
- BCL can compile a non-trivial package graph into runtime-consumable artifacts reproducibly
- package, compiler, and runtime diagnostics remain machine-readable and stable across CLI invocations
- current `moc` workflows remain available during migration without blocking the long-term platform transition

## Timeline

### Phase 1: Vision Freeze And Migration Contract

- align whitepaper, PRD, spec, README, and TODO on one target model
- define migration rules from the current local-toolchain baseline to the package/runtime/language platform
- freeze terminology:
  - `block` package
  - `moc` package
  - BCL source package/module
  - runtime host
  - resolver/lockfile

Entry gate:
- scope alignment confirmed across docs: package + runtime + language platform, no deploy-control expansion

Exit gate:
- active PRD + active spec merged
- TODO contains a dedicated platform workstream
- README and PRD index point at the unified plan

### Phase 2: Package Model And Registry Baseline

- define package manifests, identities, semantic-version rules, and lockfile shape
- split local discovery from package resolution semantics
- add local package cache and registry abstraction
- add initial publish/fetch/resolve/install commands for local + file-based registries
- landed baseline now includes the first minimal `pkg init|resolve|fetch|publish` surface, deterministic lockfile emission, file-registry layout, and migration-bridge behavior for local legacy roots

Entry gate:
- Phase 1 documents merged and accepted as the roadmap source of truth

Exit gate:
- deterministic resolution and lockfile generation exist for package graphs
- local and file-backed registry flows are test-covered
- current local-only workflows have a documented compatibility bridge

### Phase 3: Rust-Native Runtime Platform

- define stable runtime host traits and execution envelopes above concrete Rust runtimes
- separate contract/runtime semantics from local build-time dispatch glue
- add at least two host implementations to prove compatibility across Rust runtime choices
- define runtime artifact, diagnostics, and capability boundaries as platform contracts

Entry gate:
- package model and resolution semantics are stable enough to bind runtime dependencies deterministically

Exit gate:
- a shared runtime contract can execute on multiple Rust host implementations
- block execution, diagnostics, and artifact semantics no longer depend on a single reference runner shape
- current `blocks-runner-catalog` path has a documented role as compatibility bootstrap, not final architecture

### Phase 4: BCL Language Promotion

- upgrade BCL from constrained `moc` assist syntax to a first-class language definition
- add package-aware imports, versioned dependencies, richer control-flow semantics, and compiler phases
- define lowering targets:
  - runtime assembly artifacts
  - compatibility descriptors
  - migration output for legacy `moc.yaml` where needed
- keep deterministic formatter, checker, planner, graph, and explain surfaces

Entry gate:
- runtime host boundary is stable enough to receive compiled artifacts

Exit gate:
- BCL is the primary reviewed source format for new platform-native packages
- compiler output is deterministic and reproducible
- legacy `moc.yaml` parity remains available only as a migration bridge, not the final center of gravity

### Phase 5: External Adoption And Conformance

- publish reusable conformance suites for external repositories
- stabilize publish/install/upgrade/rebuild workflows
- ship package-resolution, compiler, and runtime doctor surfaces for external adopters
- document migration paths from local-only blocks/mocs into package-native platform usage

Entry gate:
- package, runtime, and compiler flows are stable inside this repository

Exit gate:
- third-party repositories can adopt the platform without depending on this repository’s workspace layout
- CI-friendly conformance and migration paths are documented and deterministic

## Open Questions

- what is the minimum package manifest that can unify `block`, `moc`, and BCL package identity without overfitting?
- should the first resolver wave support only file/local registries, or also HTTP-backed remote indexes?
- what is the lowest stable runtime host ABI that still supports `tokio`, `tauri`, and sync CLI hosts cleanly?
- when BCL becomes the primary source language, which parts of `moc.yaml` remain as explicit emitted artifacts and which become purely derived?

## Acceptance Criteria

- [x] repository scope is explicitly reframed as package + runtime + language platform
- [x] the plan defines phased work for registry/resolution, runtime hosting, and BCL promotion
- [x] README and TODO are updated in the same change so the new direction is discoverable

---

### Change Log
- 2026-03-13: Added the repository-level blocks/BCL toolchain plan with phased scope, explicit non-goals, and no-IDE first-wave boundary.
- 2026-03-13: Completed the initial toolchain wave around authoring, conformance, troubleshooting, and migration.
- 2026-03-16: Reframed the active roadmap around package management, a Rust-native runtime platform, and BCL as a first-class language.
