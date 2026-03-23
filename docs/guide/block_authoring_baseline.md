---
status: active
owner: Developer
created: 2026-03-13
updated: 2026-03-13
audience: developers
level: beginner
---

# Block Authoring Baseline

## Overview

Use this guide when you need to create or update a reusable public `block`.

This is a repository-owned authoring workflow for the R12 Phase 2 toolchain baseline. It is CLI-first and intentionally does not rely on IDE/LSP/editor integrations.

## Prerequisites

- a local `blocks/` root
- Rust toolchain available when the block uses `rust`
- the repository-level `blocks-cli` command path
- a clear decision that the work belongs in a reusable `block`, not `moc`-local glue

## Getting Started

### Step 1: Decide Whether You Need A Public Block

Create a public `block` only when all of these are true:

- the capability is reusable across more than one `moc`
- the input and output can be described as a stable contract
- success and failure can be checked quickly
- the behavior stays small and single-purpose

If the work is only product-specific glue, keep it in the `moc` entrypoint or `internal_blocks/`.

### Step 2: Scaffold The Block

Example:

```bash
cargo run -p blocks-cli -- block init blocks demo.slugify
```

Use optional flags supported by the CLI when you need to set implementation kind or target explicitly.

Expected result:

- `blocks/<block-id>/block.yaml`
- `blocks/<block-id>/README.md`
- implementation starter files
- starter evidence folders: `tests/`, `examples/`, `evaluators/`, `fixtures/`

### Step 3: Fill In The Contract

At minimum, complete:

- identity: `id`, `name`, `version`, `status`, `owner`
- capability boundary: `purpose`, `scope`, `non_goals`
- IO contract: `inputs`, `input_schema`, `outputs`, `output_schema`
- execution boundary: `implementation`, `dependencies`, `side_effects`, `timeouts`, `resource_limits`
- failure and quality: `failure_modes`, `error_codes`, `recovery_strategy`, `verification`, `evaluation`, `acceptance_criteria`

If the block will be `active`, also complete the required `debug`, `observe`, and `errors.taxonomy` fields.

### Step 4: Format The Descriptor

```bash
cargo run -p blocks-cli -- block fmt blocks/demo.slugify
```

Use formatting before review so the descriptor stays deterministic.

### Step 5: Check The Block

```bash
cargo run -p blocks-cli -- block check blocks/demo.slugify --json
```

Use `--json` when you want machine-readable diagnostics for automation or AI-driven repair loops.

### Step 6: Add Implementation And Evidence

Add the real implementation in code and then add the block-local evidence expected by the specification:

- `tests/`
- `examples/`
- `evaluators/`
- `fixtures/`

Once local evidence exists, run:

```bash
cargo run -p blocks-cli -- block test blocks/demo.slugify --json
cargo run -p blocks-cli -- block eval blocks/demo.slugify --json
cargo run -p blocks-cli -- conformance run block blocks/demo.slugify --json
```

## Common Tasks

### Update An Existing Block

1. Run `block fmt`.
2. Run `block check --json`.
3. Review any contract drift before changing implementation.

### Author A Frontend Block

Use a frontend-only block when the implementation belongs in `tauri_ts` and the target is `frontend`.

Do not use a frontend block to replace a full `frontend_app` or `frontend_lib` `moc`.

## Troubleshooting

### Problem: `block check` reports missing required fields

**Likely cause**: the descriptor still matches an older partial contract shape.  
**Solution**: fill in the missing standard contract fields before trying to use the block elsewhere.

### Problem: you are adding too much logic to one block

**Likely cause**: the capability is not actually single-purpose.  
**Solution**: split it into smaller reusable outputs or keep the composition logic in a `moc`.

### Problem: you want to use the block only in one product flow

**Likely cause**: the capability is product-local.  
**Solution**: prefer `internal_blocks/` or normal `moc` code instead of publishing a new global block.

## Best Practices

- keep the block contract explicit, small, and reviewable
- use formatting before checking in descriptor changes
- treat `block.yaml` as a descriptor, not as runtime logic
- add at least one minimal success example and one minimal failure example
- prefer direct Rust crate reuse for reusable Rust blocks

## Related Topics

- [build_moc_baseline.md](./build_moc_baseline.md)
- [bcl_authoring_baseline.md](./bcl_authoring_baseline.md)
- [conformance_workflow.md](./conformance_workflow.md)
- [BLOCKS_SPEC.md](../specs/BLOCKS_SPEC.md)
- [BLOCKS_BCL_TOOLCHAIN_PLAN.md](../prds/BLOCKS_BCL_TOOLCHAIN_PLAN.md)
