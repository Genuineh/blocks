---
status: active
owner: Developer
created: 2026-03-13
updated: 2026-03-17
audience: developers
level: intermediate
---

# Conformance Workflow

## Overview

Use this guide when you want to prove that a `block`, package graph, runtime host path, `moc`, or `moc.bcl` implementation conforms to the repository-owned toolchain conventions.

This is the public Phase 3 workflow for executable evidence, deterministic conformance, and BCL parity gating. It is CLI-first and remains independent of IDE/LSP support.

## Prerequisites

- a local repository or adoption workspace that follows the `blocks/` and `mocs/` layout
- `cargo` available for the `blocks-cli` binary
- shell access for local evidence runners such as `tests/run.sh`
- for BCL parity, both `moc.bcl` and the checked-in `moc.yaml`
- for package conformance, a package root plus any required `--provider` entries
- for runtime conformance, a block contract plus either `fixtures/*.json`, `examples/*.json`, or an explicit `--input <json-file>`

## Evidence Convention

For reusable public `block`s, the repository-owned executable evidence convention is:

- `tests/run.sh`: functional verification entrypoint
- `examples/run.sh`: runnable example entrypoint
- `evaluators/run.sh`: quality evaluation entrypoint
- `fixtures/`: sample inputs and expected outputs used by the test/eval runners

Compatibility fallback:

- if `tests/run.sh` and `examples/run.sh` are absent, `block test` can fall back to `verification.automated[]`
- if `evaluators/run.sh` is absent, `block eval` can fall back to `evaluation.commands[]`

For new work, prefer the local `run.sh` entrypoints because they are easier for AI and repository tooling to discover deterministically.

## Getting Started

### Step 1: Execute Block-Local Evidence

Run the block-level evidence commands directly:

```bash
cargo run -p blocks-cli -- block test blocks/demo.echo --json
cargo run -p blocks-cli -- block eval blocks/demo.echo --json
```

Expected result:

- `block test` executes local test/example evidence and reports fixture counts
- `block eval` executes local evaluator evidence and reports fixture counts
- both commands return stable JSON suitable for CI and AI loops

### Step 2: Run Block Conformance

```bash
cargo run -p blocks-cli -- conformance run block blocks/demo.echo --json
```

This aggregates:

- `block check`
- `block test`
- `block eval`

For the block suite to pass, the target should have non-empty `tests/`, `examples/`, `evaluators/`, and `fixtures/` evidence assets.

### Step 3: Run Package Conformance

```bash
cargo run -p blocks-cli -- conformance run package \
  packages/demo-phase2 \
  --provider file:.tmp/file-registry \
  --json
```

This runs the public Phase 2 package-resolution baseline:

- `pkg resolve`
- `pkg resolve --lock`
- a repeated lock-writing pass to prove deterministic JSON output and identical `blocks.lock` bytes

This suite is intended for adopters that only have a package root plus provider configuration and do not mirror this repository's `blocks/` and `mocs/` layout.

### Step 4: Run Runtime Conformance

```bash
cargo run -p blocks-cli -- runtime check blocks/demo.echo --json
cargo run -p blocks-cli -- conformance run runtime blocks/demo.echo --json
```

This Phase 3 runtime-host baseline proves:

- the CLI can report host capability and incompatibility reasons for `sync-cli` and `tokio-service`
- the same block contract can execute through both host profiles
- runtime output remains identical across host profiles for the same input

### Step 5: Run MOC Conformance

```bash
cargo run -p blocks-cli -- conformance run moc blocks mocs/echo-pipeline --json
```

This runs the stable `moc check` surface and, when the `moc` declares a validation flow, also runs `moc verify`.

### Step 6: Run BCL Conformance

```bash
cargo run -p blocks-cli -- conformance run bcl \
  mocs/echo-pipeline \
  --check-against mocs/echo-pipeline/moc.yaml \
  --gate-mode warn \
  --json
```

Legacy workspace-layout BCL conformance runs:

- `moc bcl check`
- `moc bcl plan`
- parity against `moc.yaml` when `--check-against` is provided

Package-root BCL conformance now also works:

```bash
cargo run -p blocks-cli -- conformance run bcl \
  packages/consumer-packaged-flow \
  --provider workspace:packages \
  --gate-mode off \
  --json
```

This package-aware path runs:

- `bcl check`
- `bcl graph`
- `bcl build`
- optional parity against `moc.yaml` when `--check-against` is provided

## BCL Gate Modes

The repository-owned BCL gate supports three modes:

- `off`: skip parity gating entirely; this is the rollback path
- `warn`: parity drift is reported but does not fail conformance
- `error`: parity drift fails conformance and repository checks

The environment fallback is:

```bash
BLOCKS_BCL_GATE_MODE=warn
```

`./scripts/repo_check.sh` reads this environment variable. As of 2026-03-13, the repository default is `warn`.

The repository now also exposes the same baseline in GitHub Actions through [`.github/workflows/repo-check.yml`](../../.github/workflows/repo-check.yml).

Promotion rule:

- the repository should only change the default from `warn` to `error` after at least 14 consecutive calendar days of green parity checks across all opted-in trial `moc`s

## Common Tasks

### Adopt The Toolchain In Another Repository

1. if you are only adopting package resolution, create a package root and required registry/provider configuration
2. if you are adopting reusable blocks or mocs, mirror the `blocks/` and `mocs/` layout
3. expose block-local evidence through `tests/run.sh`, `examples/run.sh`, `evaluators/run.sh`, and `fixtures/`
4. run `conformance run package` for package-resolution and lockfile verification
5. run `runtime check` and `conformance run runtime` for reusable Rust blocks that should stay portable across host profiles
6. run `conformance run block` for reusable public blocks
7. run `conformance run moc` for delivery units
8. run `conformance run bcl` for opted-in BCL sources, using `--check-against` when parity to `moc.yaml` is still part of the migration gate

### Roll Back The BCL Gate

Use this when parity drift should not block the repository temporarily:

```bash
BLOCKS_BCL_GATE_MODE=off ./scripts/repo_check.sh
```

This preserves existing `moc.yaml` runtime authority and disables the BCL parity gate without changing `moc run` / `moc verify` behavior.

## Troubleshooting

### Problem: `block test` says no executable evidence is configured

**Likely cause**: there is no local `tests/run.sh` or `examples/run.sh`, and no usable `verification.automated[]` fallback.  
**Solution**: add the repository-owned local runners and keep fixtures next to them.

### Problem: block conformance fails on empty evidence directories

**Likely cause**: the block has placeholder folders but no real evidence assets yet.  
**Solution**: add at least one real runner/file to `tests/`, `examples/`, `evaluators/`, and `fixtures/`.

### Problem: BCL conformance fails only in `error` mode

**Likely cause**: `moc.bcl` and `moc.yaml` have drifted apart.  
**Solution**: fix parity with `moc bcl emit --check-against`, or temporarily switch to `BLOCKS_BCL_GATE_MODE=warn` while repairing the drift.

## Best Practices

- prefer local block evidence runners over remote or product-specific test commands
- use `conformance run package` as the minimum external-adopter entrypoint when only package resolution is in scope
- use `runtime check` before `conformance run runtime` so incompatibility reasons are visible without digging into execution failures
- keep fixture data small, reviewable, and checked in
- use `conformance run` as the public CI surface instead of stitching private scripts together
- keep CI wired to the repository-owned [`.github/workflows/repo-check.yml`](../../.github/workflows/repo-check.yml) so local and hosted verification paths stay identical
- keep BCL parity in `warn` mode until trial `moc`s stay green for the full stability window
- use `off` only as a short rollback path, not as the steady-state default

## Related Topics

- [block_authoring_baseline.md](./block_authoring_baseline.md)
- [package_registry_baseline_workflow.md](./package_registry_baseline_workflow.md)
- [build_moc_baseline.md](./build_moc_baseline.md)
- [bcl_authoring_baseline.md](./bcl_authoring_baseline.md)
- [BLOCKS_BCL_TOOLCHAIN_SPEC.md](../specs/BLOCKS_BCL_TOOLCHAIN_SPEC.md)
- [BCL_MOC_MVP_SPEC.md](../specs/BCL_MOC_MVP_SPEC.md)
