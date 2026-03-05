---
name: Blocks CLI Decoupling Phase 3
description: Replace the handwritten catalog dispatch table with build-time generated glue while keeping the catalog manifest as the single manual registration surface.
status: completed
created: 2026-03-04
author: Developer
---

## Development Log

### Blocks CLI Decoupling Phase 3

**Status**: Completed

**Started**: 2026-03-04 13:30
**Updated**: 2026-03-04 14:10
**Owner**: Developer

#### Objective
Implement Phase 3 by reducing manual registration in `blocks-runner-catalog`: keep `Cargo.toml` as the only handwritten registration surface, validate sibling `block.yaml` metadata at build time, and generate deterministic dispatch glue without changing CLI command wiring.

#### Progress
- [x] Added shared catalog codegen support plus `build.rs` to parse local `block-*` dependencies and emit generated dispatch glue into `OUT_DIR`.
- [x] Replaced the handwritten catalog `match` table with an `include!` of generated glue while keeping the unknown-block fallback string unchanged.
- [x] Added coverage for generated registration ordering and invalid metadata failure using the same codegen path as the build script.
- [x] Updated `skills/create-block.md`, `docs/TODO.md`, and the Phase 3 PRD notes to point contributors at the catalog manifest.
- [x] Ran targeted cargo tests and recorded final status.

#### Blockers
- None.

#### Records
| Date | Action | Description |
|------|--------|-------------|
| 2026-03-04 | Created | Initialized the Phase 3 development log. |
| 2026-03-04 | Updated | Added build-time catalog code generation driven by `crates/blocks-runner-catalog/Cargo.toml` and validated sibling `block.yaml` files through `blocks-contract`. |
| 2026-03-04 | Updated | Replaced the handwritten dispatch table with generated glue, preserving the exact unknown-block fallback string. |
| 2026-03-04 | Completed | Ran targeted cargo tests, updated TODO/PRD/skill docs, and marked Phase 3 complete. |
