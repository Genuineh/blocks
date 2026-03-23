# PRDS

This directory contains active plans, architecture notes, and design details that drive current implementation.

- `MVP_PLAN.md`: phased MVP scope and delivery boundaries.
- `RUST_WORKSPACE_ARCHITECTURE.md`: current Rust workspace architecture baseline.
- `BLOCKS_CLI_DECOUPLING_PLAN.md`: phased plan for moving runnable block wiring out of `blocks-cli`.
- `GREETING_PROOF_SLICE_PLAN.md`: approved plan for the minimal real frontend/backend proof slice under the `moc` model.
- `BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md`: baseline plan for mandatory block debuggability and observability capabilities.
- `ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md`: short-cycle plan for contract/runtime/cli architecture debt reduction.
- `R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md`: R10 Phase 1 minimal landing plan for runtime boundary unification, moc-level diagnostics ownership, and taxonomy-aware error mapping.
- `BCL_MOC_ASSIST_PLAN.md`: phased plan for establishing BCL as a moc authoring/validation assist layer without replacing moc runtime authority.
- `BLOCKS_BCL_TOOLCHAIN_PLAN.md`: active unified plan for turning this repository into the `blocks` package, runtime, and BCL language platform, including registry/resolution, Rust-native runtime hosting, and BCL promotion.
- `BLOCKS_PHASE2_PACKAGE_REGISTRY_PLAN.md`: execution-focused plan for the landed Phase 2 package manifest, lockfile, provider precedence, and file-registry baseline.

When a plan is replaced or no longer active, move it to `docs/archive/` and update links in `README.md` and `docs/TODO.md`.
