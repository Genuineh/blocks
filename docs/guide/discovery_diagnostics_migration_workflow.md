---
status: active
owner: Developer
created: 2026-03-13
updated: 2026-03-13
audience: developers
level: intermediate
---

# Discovery, Diagnostics, And Migration Workflow

## Overview

Use this guide when you need the R12 Phase 4 public toolchain loop:

- discover reusable local blocks with a stable catalog
- inspect `block` / `moc` health with repair-oriented summaries
- visualize and explain BCL assembly
- compare descriptor compatibility before change review
- preview or apply baseline upgrades without IDE support

This workflow is CLI-first and keeps `moc.yaml` as runtime authority.

## Prerequisites

- local repository layout with `blocks/` and `mocs/`
- `cargo` available for the `blocks-cli` binary
- checked-in descriptors you want to inspect or migrate

## Step 1: Discover Available Blocks

Export the local catalog:

```bash
cargo run -p blocks-cli -- catalog export blocks --json
```

Search the catalog:

```bash
cargo run -p blocks-cli -- catalog search blocks echo --json
```

Current Phase 4 behavior:

- catalog entries are local-block oriented in the first wave
- each entry exposes implementation kind/target, purpose, schema summaries, side effects, and evidence presence
- optional filters are available through `--kind`, `--target`, and `--status`

## Step 2: Inspect Repair-Oriented Health

For a reusable block:

```bash
cargo run -p blocks-cli -- block doctor blocks blocks/demo.echo --json
```

For a delivery moc:

```bash
cargo run -p blocks-cli -- moc doctor blocks mocs/echo-pipeline --json
```

`doctor` surfaces focus on what to do next, not only what failed. They summarize:

- existing `check` errors and warnings
- evidence or launcher gaps
- latest available diagnostics
- protocol health for dependent mocs
- explicit next actions for humans and AI agents

## Step 3: Visualize And Explain BCL

Inspect the lowered assembly graph:

```bash
cargo run -p blocks-cli -- moc bcl graph blocks mocs/echo-pipeline --json
```

Ask for a repair-oriented explanation:

```bash
cargo run -p blocks-cli -- moc bcl explain blocks mocs/echo-pipeline --json
```

Expected behavior:

- `graph` exports nodes and edges for uses, dependencies, protocols, flows, steps, and binds
- `explain` returns success summaries for valid BCL and structured issues for validation/planning failures
- `explain --json` returns error JSON on invalid sources so CI and AI loops can treat it as a failing step

## Step 4: Compare Compatibility Before Merging

Compare two block descriptors:

```bash
cargo run -p blocks-cli -- compat block before/block.yaml after/block.yaml --json
```

Compare two moc descriptors:

```bash
cargo run -p blocks-cli -- compat moc before/moc.yaml after/moc.yaml --json
```

Compare two BCL sources through emitted manifests:

```bash
cargo run -p blocks-cli -- compat bcl blocks before/moc.bcl after/moc.bcl --json
```

Current classification:

- `breaking`: required-field, type, protocol, or structural changes that can invalidate callers
- `compatible`: additive or loosened changes
- compatibility is semantic-first for schemas and protocols; it is not a version resolver

## Step 5: Preview Or Apply Baseline Upgrades

Preview a block migration to the current repository-owned baseline:

```bash
cargo run -p blocks-cli -- upgrade block blocks/demo.echo --json
```

Apply it:

```bash
cargo run -p blocks-cli -- upgrade block blocks/demo.echo --write
```

Also supported:

- `upgrade moc <moc-root|moc.yaml>`
- `upgrade bcl <moc-root|moc.bcl>`

Current rule-set:

```bash
--rule-set r12-phase4-baseline
```

Upgrade behavior in the first wave:

- default mode is preview, not write
- descriptors are canonicalized to the current formatter output
- block/moc upgrades also report missing baseline directories that should exist under the current toolchain

## Repository Gate

`./scripts/repo_check.sh` now exercises Phase 4 surfaces directly:

- `catalog export`
- `block doctor`
- `moc doctor`
- `moc bcl graph`
- `compat`
- `upgrade`

This means external adopters can reuse the same CLI entrypoints instead of writing custom glue first.

The repository-owned GitHub Actions baseline is [`.github/workflows/repo-check.yml`](../../.github/workflows/repo-check.yml), which runs the same script and enables CI-only frontend-host dependency prefetching before the offline Tauri probes.

## Related Topics

- [block_authoring_baseline.md](./block_authoring_baseline.md)
- [build_moc_baseline.md](./build_moc_baseline.md)
- [bcl_authoring_baseline.md](./bcl_authoring_baseline.md)
- [conformance_workflow.md](./conformance_workflow.md)
- [BLOCKS_BCL_TOOLCHAIN_SPEC.md](../specs/BLOCKS_BCL_TOOLCHAIN_SPEC.md)
