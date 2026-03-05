# Blocks CLI Decoupling Plan

## Problem

`blocks-cli` currently owns two separate responsibilities:

- user-facing command parsing (`list`, `show`, `run`, `moc ...`)
- runnable Rust block registration and dispatch

Today that registration is hard-wired inside `CliBlockRunner`, so each new runnable Rust block requires:

- adding a direct crate dependency to `crates/blocks-cli/Cargo.toml`
- adding a new `match` arm in `crates/blocks-cli/src/main.rs`

This makes `blocks-cli` the scaling bottleneck for runtime integration and weakens the intended registry-driven model.

## Goal

Keep `blocks-cli` as the command entrypoint, but stop making it the place where runnable block crates are individually wired.

Success means:

- adding a new runnable Rust block no longer requires editing CLI command code
- block execution registration has a dedicated ownership boundary
- the CLI depends on a stable runner interface instead of per-block wiring details

## Target Design

Introduce a dedicated runner catalog layer between the CLI and concrete block crates.

Planned shape:

- `blocks-cli`: parse commands, load manifests/registry, call a runner provider
- `blocks-runner-catalog` (new crate): own the mapping from `block_id` to executable Rust block implementation
- block crates: continue exposing `run(&Value) -> Result<Value, BlockExecutionError>`

The CLI should depend on one catalog crate, not on every runnable block crate directly.

## Phase Plan

### Phase 1: Extract Wiring Boundary

- Create a new crate (for example `crates/blocks-runner-catalog`).
- Move `CliBlockRunner` and the `match block_id` dispatch table out of `crates/blocks-cli/src/main.rs`.
- Keep the dispatch logic unchanged for now; this phase is about ownership, not behavior.

Acceptance:

- `blocks-cli` no longer imports concrete block crates directly.
- command behavior stays identical.

### Phase 2: Stabilize Runner Contract

- Expose a small public constructor from the catalog crate, such as `default_block_runner()`.
- Keep `blocks-cli` talking only to the `BlockRunner` trait boundary from `blocks-runtime`.
- Add focused tests around unknown-block handling and one or two known registered blocks at the catalog layer.

Acceptance:

- runtime registration tests live with the catalog
- CLI tests stop needing to assert dispatch details owned by the catalog

Implementation note (2026-03-04):

- `blocks-runner-catalog` now hides the concrete catalog runner and exposes `default_block_runner() -> impl BlockRunner`.
- `blocks-cli` only threads a `&impl BlockRunner` through `blocks run` and the `moc verify` execution helper; `blocks-runtime` remained unchanged.
- The unknown-block fallback and the `core.http.get` runtime smoke (through `Runtime::execute` and contract validation) now live in `blocks-runner-catalog`.

### Phase 3: Reduce Manual Registration

- Decide whether registration stays as one explicit catalog list or moves to generated glue.
- If generation is adopted, base it on repository metadata already available in block descriptors, but keep generation output deterministic and reviewable.
- Do not introduce dynamic runtime loading yet; keep the runtime simple until there is a real need.

Acceptance:

- adding a runnable Rust block touches either one catalog file or one generator input, not CLI command code

Implementation note (2026-03-04):

- `crates/blocks-runner-catalog/Cargo.toml` is now the single manual registration surface for runnable Rust blocks.
- `crates/blocks-runner-catalog/build.rs` parses the catalog crate’s own `block-*` path dependencies, resolves each sibling `block.yaml`, validates it through `blocks-contract`, and generates deterministic dispatch glue into `OUT_DIR`.
- `crates/blocks-runner-catalog/src/lib.rs` now includes the generated dispatch glue and keeps the unknown-block fallback string handwritten and unchanged.
- The build script emits `cargo:rerun-if-changed` for the catalog `Cargo.toml` and every resolved `block.yaml`, and invalid block metadata fails the build instead of being skipped.

## Non-Goals

- No plugin system in this phase
- No runtime dynamic linking
- No change to the `blocks-runtime::BlockRunner` trait unless a concrete need appears during extraction
- No change to `moc` manifest behavior

## Risks

- Moving the runner without changing behavior can still break command wiring if tests are too CLI-centric.
- Over-abstracting too early would add ceremony without reducing real coupling.

Mitigation:

- make Phase 1 a pure extraction
- keep the runtime trait unchanged during the first pass
- only consider generated registration after the ownership boundary is clean

## Recommended Next Implementation Slice

The smallest safe next change is:

1. Add `crates/blocks-runner-catalog`.
2. Move the current `CliBlockRunner` implementation into that crate.
3. Replace the CLI-local runner with a call to the catalog crate.
4. Keep the existing hard-coded dispatch table unchanged until that extraction is stable.
