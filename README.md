# blocks

`blocks` is a software building-block model designed for AI-driven development. Its goal is not to replace existing programming languages, frameworks, or libraries, but to add a layer of composable, verifiable capabilities on top of them so AI can assemble, validate, and ship higher-quality results with more consistency.

The active repository direction is now broader than a local validation toolchain: this repository is intended to become the `blocks` package, runtime, and language platform. That means package discovery and dependency resolution, a Rust-native runtime platform boundary, and `BCL` as a first-class language are now part of the planned architecture.

In modern software production, AI struggles to deliver consistently high-quality output not only because large language models are probabilistic, but also because much of the existing toolchain was not designed around how AI understands, invokes, and validates systems. Those tools work well for human engineers, but for AI they often impose high comprehension cost, unclear invocation paths, fuzzy behavioral boundaries, and expensive verification loops.

The core claim of `blocks` is simple: if you want AI to produce high-quality products, every minimal component it uses must first be stable, verifiable, and evaluable. High-quality systems only emerge from high-quality building blocks.

## Core Definition

A `block` is the smallest usable production unit for AI. It is not a broad framework abstraction. It is a component designed around one simple, explicit, stable output capability.

A valid `block` should satisfy the following:

- AI can understand what it does quickly.
- AI can invoke it accurately without relying on vague inference.
- AI can verify whether it works correctly with low effort.
- Humans and machines can evaluate the quality of its output.
- It can compose cleanly with other `block`s to form more complex system capabilities.

## Why `blocks`

Current AI instability in software production typically appears in three areas:

- `LLM` output is probabilistic, so the same task can produce inconsistent results.
- Existing frameworks and libraries are mostly optimized for human developers, not AI invocation and validation patterns.
- AI usage strategies vary across languages, frameworks, and requirement constraints, which introduces understanding drift and implementation drift.

That means even with strong core `tool` capabilities, AI can still become unstable when deciding how to use a framework correctly, how to choose the right abstraction, and how to verify results efficiently.

## The `blocks` Approach

`blocks` does not ask AI to directly master entire complex systems. It asks us to first decompose those systems into controllable, stable minimal parts:

1. Break requirements into minimal capability units with independent responsibility.
2. Define clear inputs, outputs, constraints, and verification methods for each unit.
3. Make those units stable first, then let AI assemble, coordinate, and choose between them.
4. When a new need appears, add a new qualified `block` instead of expanding a fuzzy problem surface.

In other words, AI should make composition decisions within a stable set of working parts, not perform high-risk exploration inside complex frameworks with unclear boundaries and behavior.

## Design Principles

- `AI-first`: component design prioritizes AI understanding, invocation, and verification paths.
- `Verifiable`: every component must include an explicit verification path.
- `Evaluable`: outputs must be objectively assessable, not merely subjectively "good enough."
- `Composable`: components should combine with low ambiguity into higher-level capabilities.
- `Minimal`: each component abstracts the simplest, most stable output, not an over-generalized large interface.
- `Replaceable`: components should be independently upgradable or replaceable without breaking the whole system.

## What `blocks` Solves

`blocks` aims to establish a new software production foundation that gives AI more determinism in the following areas:

- lower invocation ambiguity
- higher implementation stability
- faster result verification
- clearer quality boundaries
- more sustainable capability accumulation

## A Simple Example

Suppose a system requires 10 cooperating parts:

- `blocks` does not ask AI to reinvent those 10 capabilities from scratch.
- `blocks` requires those 10 parts to be stable, verifiable, and independently deliverable.
- AI's role is to decide how to combine, order, and coordinate them to meet a goal.

If an eleventh requirement appears, the right move is not to force AI to improvise on top of unstable capabilities. The right move is to build an eleventh qualified `block` and then add it to the composable system.

## What This Is Not

- It is not a replacement for existing programming languages.
- It is not a rejection of the value of existing frameworks and libraries.
- It does not require rebuilding everything from scratch.
- It does not try to eliminate `LLM` probabilistic behavior.

The goal of `blocks` is to add a layer of engineering constraints and verifiable abstractions on top of the existing software ecosystem that is better suited to AI.

## Repository Documents

- [docs/TODO.md](./docs/TODO.md): current backlog, priorities, and near-term execution order.
- [docs/prds/MVP_PLAN.md](./docs/prds/MVP_PLAN.md): the current minimal MVP plan, focused on the `moc` model, phases, and implementation path.
- [docs/prds/RUST_WORKSPACE_ARCHITECTURE.md](./docs/prds/RUST_WORKSPACE_ARCHITECTURE.md): architecture sketch for the current Rust workspace under the `moc` model.
- [docs/prds/BLOCKS_CLI_DECOUPLING_PLAN.md](./docs/prds/BLOCKS_CLI_DECOUPLING_PLAN.md): plan for decoupling `blocks-cli` runtime wiring and moving executable block registration out of the CLI command layer.
- [docs/prds/GREETING_PROOF_SLICE_PLAN.md](./docs/prds/GREETING_PROOF_SLICE_PLAN.md): minimal real frontend/backend proof-slice plan, defining the delivery boundaries of `greeting-api-service` and `greeting-panel-web`.
- [docs/prds/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md](./docs/prds/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md): foundation plan for block debuggability and observability, defining a unified diagnostics baseline.
- [docs/prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md](./docs/prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md): current 1-2 week architecture debt reduction plan focused on `contract`, `runtime`, and `cli`.
- [docs/prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md](./docs/prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md): minimal R10 Phase 1 plan focused on unifying the run/verify runtime boundary, moc-level diagnostic ownership, and taxonomy mapping.
- [docs/prds/BCL_MOC_ASSIST_PLAN.md](./docs/prds/BCL_MOC_ASSIST_PLAN.md): BCL establishment plan, defining BCL strictly as a `moc` authoring and validation assist layer rather than a runtime delivery boundary.
- [docs/prds/BLOCKS_BCL_TOOLCHAIN_PLAN.md](./docs/prds/BLOCKS_BCL_TOOLCHAIN_PLAN.md): active unified platform plan covering package management, Rust-native runtime hosting, and BCL promotion from MVP assist syntax to first-class language.
- [docs/prds/BLOCKS_PHASE2_PACKAGE_REGISTRY_PLAN.md](./docs/prds/BLOCKS_PHASE2_PACKAGE_REGISTRY_PLAN.md): the concrete Phase 2 execution plan for package manifests, lockfiles, provider precedence, migration bridge, and the first `blocks pkg` baseline.
- [docs/guide/README.md](./docs/guide/README.md): entry point for usage guides and contribution workflows.
- [docs/guide/blocks_bcl_toolchain_handbook.md](./docs/guide/blocks_bcl_toolchain_handbook.md): the recommended single-guide entry point for using the full `block` / `moc` / BCL toolchain end to end.
- [docs/guide/block_authoring_baseline.md](./docs/guide/block_authoring_baseline.md): public authoring-baseline workflow for scaffolding, formatting, and checking reusable `block`s.
- [docs/guide/build_moc_baseline.md](./docs/guide/build_moc_baseline.md): public authoring-baseline workflow for scaffolding, formatting, and checking `moc`s.
- [docs/guide/bcl_authoring_baseline.md](./docs/guide/bcl_authoring_baseline.md): public authoring-baseline workflow for scaffolding, formatting, and checking `moc.bcl`.
- [docs/guide/conformance_workflow.md](./docs/guide/conformance_workflow.md): public conformance workflow for executable block evidence, `conformance run`, and BCL gate rollback.
- [docs/guide/discovery_diagnostics_migration_workflow.md](./docs/guide/discovery_diagnostics_migration_workflow.md): public Phase 4 workflow for catalog discovery, doctor/graph/explain, compat, and upgrade.
- [docs/guide/package_registry_baseline_workflow.md](./docs/guide/package_registry_baseline_workflow.md): public Phase 2 workflow for package scaffolding, resolution, file-registry publish/fetch, and migration-bridge usage.
- [docs/guide/bcl_mvp_workflow.md](./docs/guide/bcl_mvp_workflow.md): practical guide for the current BCL MVP workflow, including validate/plan/emit/parity usage and authority boundaries.
- [docs/decisions/README.md](./docs/decisions/README.md): index of architecture decision records.
- [docs/decisions/001-enforce-contract-runtime-boundary.md](./docs/decisions/001-enforce-contract-runtime-boundary.md): decision record for strong contract enforcement, unified runtime boundaries, and CLI layering.
- [docs/decisions/002-r10-phase1-runtime-observability-boundary.md](./docs/decisions/002-r10-phase1-runtime-observability-boundary.md): minimal R10 Phase 1 decision fixing unified runtime observability boundaries and controlled fallback behavior.
- [docs/decisions/003-bcl-assists-moc-not-runtime.md](./docs/decisions/003-bcl-assists-moc-not-runtime.md): BCL MVP boundary decision fixing "assist `moc`, do not replace runtime authority."
- [docs/archive/README.md](./docs/archive/README.md): entry point for archived documents and archive rules.
- [docs/whitepapers/WHITEPAPER.md](./docs/whitepapers/WHITEPAPER.md): the core `blocks` whitepaper, explaining why this AI-oriented building-block model is needed.
- [docs/whitepapers/DEVELOPMENT_WHITEPAPER.md](./docs/whitepapers/DEVELOPMENT_WHITEPAPER.md): development whitepaper defining how to deliver projects using `blocks` capabilities.
- [docs/specs/BLOCKS_SPEC.md](./docs/specs/BLOCKS_SPEC.md): the `block` specification, defining the structure, contracts, verification, and quality requirements for public capability units.
- [docs/specs/MOC_SPEC.md](./docs/specs/MOC_SPEC.md): the `moc` specification, defining delivery-unit types, structure, descriptor files, and protocol boundaries.
- [docs/specs/GREETING_PROOF_SLICE_SPEC.md](./docs/specs/GREETING_PROOF_SLICE_SPEC.md): technical specification for the greeting proof slice, defining interfaces and verification boundaries.
- [docs/specs/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_SPEC.md](./docs/specs/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_SPEC.md): technical specification for block debuggability and observability foundations, including diagnostic events, artifacts, and CLI boundaries.
- [docs/specs/ARCHITECTURE_REFACTOR_SPEC_2026Q1.md](./docs/specs/ARCHITECTURE_REFACTOR_SPEC_2026Q1.md): technical specification for `contract`/`runtime`/`cli` architecture refactoring.
- [docs/specs/R10_PHASE1_RUNTIME_BOUNDARY_SPEC.md](./docs/specs/R10_PHASE1_RUNTIME_BOUNDARY_SPEC.md): function-level R10 Phase 1 spec covering shared run/verify execution boundaries, `moc diagnose` corrections, and `error_id` mapping.
- [docs/specs/BCL_MOC_MVP_SPEC.md](./docs/specs/BCL_MOC_MVP_SPEC.md): minimal BCL technical specification covering grammar, semantic validation, CLI contract, and phased rollout gates.
- [docs/specs/BLOCKS_BCL_TOOLCHAIN_SPEC.md](./docs/specs/BLOCKS_BCL_TOOLCHAIN_SPEC.md): technical specification for the package/runtime/language platform direction, including resolver/lockfile, runtime host contracts, compiler flows, and migration aliases.
- [docs/specs/BLOCKS_PACKAGE_MODEL_SPEC.md](./docs/specs/BLOCKS_PACKAGE_MODEL_SPEC.md): Phase 2 normative package manifest, lockfile, and authority rules.
- [docs/specs/BLOCKS_REGISTRY_BASELINE_SPEC.md](./docs/specs/BLOCKS_REGISTRY_BASELINE_SPEC.md): Phase 2 provider precedence, file-registry layout, and fetch error taxonomy.
- [docs/whitepapers/BLOCKS_LANGUAGE_WHITEPAPER.md](./docs/whitepapers/BLOCKS_LANGUAGE_WHITEPAPER.md): BCL whitepaper defining the language model, basic syntax, compiler shape, and output model.
- [mocs/echo-pipeline/README.md](./mocs/echo-pipeline/README.md): current minimal `moc` example whose backend directly depends on the `demo.echo` Rust crate.
- [mocs/hello-pipeline/README.md](./mocs/hello-pipeline/README.md): current minimal `moc` example whose backend directly depends on file-oriented Rust block crates.
- [mocs/hello-world-console/README.md](./mocs/hello-world-console/README.md): free `moc.main` example that combines `hello-message-lib` and `core.console.write_line`.
- [mocs/hello-message-lib/README.md](./mocs/hello-message-lib/README.md): minimal `rust_lib` moc example that also provides a cross-`moc` protocol sample.
- [mocs/hello-panel-lib/README.md](./mocs/hello-panel-lib/README.md): minimal `frontend_lib` moc example with a unified `moc dev` preview entry.
- [mocs/counter-panel-web/README.md](./mocs/counter-panel-web/README.md): minimal interactive `frontend_app` moc example with a counter UI, preview page, and real Tauri host.
- [mocs/hello-panel-web/README.md](./mocs/hello-panel-web/README.md): minimal `frontend_app` moc example demonstrating the Tauri + TypeScript boundary and local preview entry.
- [mocs/greeting-api-service/README.md](./mocs/greeting-api-service/README.md): minimal `backend_app(service)` moc example exposing a real HTTP API contract.
- [mocs/greeting-panel-web/README.md](./mocs/greeting-panel-web/README.md): minimal real-data `frontend_app` moc example that fetches and renders a greeting from the backend.
- [skills/create-block.md](./skills/create-block.md): standard process for creating a new block.
- [skills/build-moc.md](./skills/build-moc.md): current skill guide for building a `moc`.

## BCL Trial Workflow

The BCL MVP core loop is now available for selected trial mocs while keeping `moc.yaml` as runtime authority. Repository gate rollout, rollback, and the surrounding Phase 4 toolchain surfaces are now available through the public CLI.

Example using `echo-pipeline`:

```bash
mkdir -p .tmp
cargo run -p blocks-cli -- bcl check mocs/echo-pipeline --json
cargo run -p blocks-cli -- moc bcl plan blocks mocs/echo-pipeline/moc.bcl --json
cargo run -p blocks-cli -- moc bcl emit blocks mocs/echo-pipeline/moc.bcl --out .tmp/echo-pipeline.generated.yaml --check-against mocs/echo-pipeline/moc.yaml
```

Current trial mocs:

- `mocs/echo-pipeline`: flow-heavy parity proof
- `mocs/greeting-panel-web`: protocol-heavy parity proof
