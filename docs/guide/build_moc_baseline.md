---
status: active
owner: Developer
created: 2026-03-13
updated: 2026-03-13
audience: developers
level: beginner
---

# Build MOC Baseline

## Overview

Use this guide when you need to author a new `moc` from existing `block`s and repository-supported contracts.

This guide covers the R12 Phase 2 authoring baseline only: scaffold, format, check, and the handoff into `run`/`verify`/`dev`. It is CLI-first and does not depend on IDE/LSP/editor support.

## Prerequisites

- a local `blocks/` root
- a local `mocs/` root
- at least one existing block or a clear plan for which blocks the `moc` will use
- the `blocks-cli` command path

## Getting Started

### Step 1: Choose The Smallest Valid MOC Type

Pick exactly one type:

- `rust_lib`
- `frontend_lib`
- `frontend_app`
- `backend_app`

If you choose `backend_app`, you also need `backend_mode: console | service`.

Do not use one `moc` to represent multiple delivery shapes.

### Step 2: Scaffold The MOC

Example:

```bash
cargo run -p blocks-cli -- moc init mocs hello-service --type backend_app --backend-mode service --language rust
```

Expected result:

- `mocs/<moc-id>/moc.yaml`
- `mocs/<moc-id>/README.md`
- starter source layout for the declared `moc` type

### Step 3: Declare The Descriptor Boundary

At minimum, complete:

- `id`, `name`, `type`, `language`, `entry`
- `backend_mode` when `type=backend_app`
- `public_contract`
- `uses.blocks`
- `uses.internal_blocks`
- `depends_on_mocs`
- `protocols`
- `verification`
- `acceptance_criteria`

Remember:

- `moc.yaml` is a descriptor and validation boundary
- real behavior still belongs in the `moc` launcher code

### Step 4: Format The Descriptor

```bash
cargo run -p blocks-cli -- moc fmt mocs/hello-service/moc.yaml
```

### Step 5: Check The MOC

```bash
cargo run -p blocks-cli -- moc check blocks mocs/hello-service/moc.yaml --json
```

This should confirm:

- the descriptor is structurally valid
- the layout exists
- `uses.blocks` and validation flow steps are consistent
- cross-`moc` protocol declarations match when dependencies are declared

### Step 6: Implement The Real Entry

After `check` passes, implement the real launcher code under the `entry` path and use:

- `moc run` for real runtime entry or preview entry
- `moc verify` for validation-flow execution
- `moc dev` for local library/frontend preview workflows
- `conformance run moc` for the public conformance surface once the descriptor is stable

## Common Tasks

### Build A Descriptor-Only MOC First

This is acceptable when you need to establish structure before the launcher is fully implemented.

Use `moc check` first, then add the launcher before expecting `moc run` to work.

### Add Private Product Logic

When the logic is stable but not yet globally reusable:

- place it in `internal_blocks/`
- keep the contract clear
- promote it to a public `block` later only if reuse appears across multiple `moc`s

### Add A Validation Flow

Use `verification.entry_flow` and `verification.flows` only when a serial validation flow helps.

Keep the flow:

- explicit
- serial
- minimal
- fully bound

## Troubleshooting

### Problem: `moc check` fails on protocol mismatch

**Likely cause**: local `depends_on_mocs` and target `protocols` do not agree.  
**Solution**: align `name`, `channel`, `input_schema`, and `output_schema` on both sides.

### Problem: `moc run` refuses to execute

**Likely cause**: the `moc` does not yet have a real launcher or preview path.  
**Solution**: add the real launcher first, or use `moc verify` if you only want to execute the validation flow.

### Problem: too much logic is being pushed into the descriptor

**Likely cause**: the descriptor is being treated like runtime code.  
**Solution**: move real behavior back into the launcher and keep `moc.yaml` as descriptor + validation boundary.

## Best Practices

- choose the smallest valid `moc` type
- keep `uses.blocks` explicit and current
- use `internal_blocks/` for private stable logic
- run `moc fmt` before `moc check`
- treat `moc verify` as validation-flow execution, not as the main runtime

## Related Topics

- [block_authoring_baseline.md](./block_authoring_baseline.md)
- [bcl_authoring_baseline.md](./bcl_authoring_baseline.md)
- [conformance_workflow.md](./conformance_workflow.md)
- [MOC_SPEC.md](../specs/MOC_SPEC.md)
- [BLOCKS_BCL_TOOLCHAIN_PLAN.md](../prds/BLOCKS_BCL_TOOLCHAIN_PLAN.md)
