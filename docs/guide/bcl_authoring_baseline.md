---
status: active
owner: Developer
created: 2026-03-13
updated: 2026-03-13
audience: developers
level: intermediate
---

# BCL Authoring Baseline

## Overview

Use this guide when you want to author or maintain `moc.bcl` as the structured source for a `moc`.

This guide covers the R12 Phase 2 authoring baseline: scaffold, format, check, and then the existing `validate -> plan -> emit -> check-against` loop. It remains CLI-first and does not rely on IDE/LSP/editor support.

## Prerequisites

- a `moc` that already has a valid `moc.yaml`, or a clear target `moc` shape
- access to the local `blocks/` root
- the `blocks-cli` command path
- understanding that `moc.yaml` remains the runtime authority

## Current Authority Rule

- `moc.bcl` is an authoring and validation source
- `moc.yaml` remains the runtime authority
- BCL does not replace runtime launchers, deploy systems, or product-level orchestration

## Getting Started

### Step 1: Scaffold The BCL Source

You can start from an existing `moc.yaml` or from a target moc identity.

Example:

```bash
cargo run -p blocks-cli -- moc bcl init mocs/hello-service/moc.yaml
```

Expected result:

- a starter `moc.bcl` next to the target `moc.yaml`
- a descriptor shape aligned with the current MVP grammar

### Step 2: Format The BCL Source

```bash
cargo run -p blocks-cli -- moc bcl fmt mocs/hello-service/moc.bcl
```

Use formatting before review and before comparing emitted YAML.

### Step 3: Check The BCL Source

```bash
cargo run -p blocks-cli -- moc bcl check blocks mocs/hello-service/moc.bcl --json
```

Treat `check` as the authoring-baseline entrypoint for syntax, semantic, and protocol correctness.

### Step 4: Inspect The Plan

```bash
cargo run -p blocks-cli -- moc bcl plan blocks mocs/hello-service/moc.bcl --json
```

Use this when you want to inspect the lowered `moc` shape before emission.

### Step 5: Emit And Compare

```bash
mkdir -p .tmp
cargo run -p blocks-cli -- moc bcl emit blocks mocs/hello-service/moc.bcl --out .tmp/hello-service.generated.yaml --check-against mocs/hello-service/moc.yaml
```

This proves that the authored BCL still matches the checked-in runtime descriptor after canonical normalization.

### Step 6: Run The Public BCL Conformance Gate

```bash
cargo run -p blocks-cli -- conformance run bcl \
  blocks \
  mocs/hello-service \
  --check-against mocs/hello-service/moc.yaml \
  --gate-mode warn \
  --json
```

Use `warn` while parity is stabilizing, `error` when drift must block the repository, and `off` as the documented rollback switch.

## Common Tasks

### Author A Descriptor-Only BCL

This is valid when the target `moc` does not need a validation flow.

Keep:

- `uses`
- `depends_on_mocs`
- `protocols`
- `verification.commands`
- `accept`

Add validation flow content only when you really need serial flow validation.

### Add A Validation Flow

When you add a flow:

- declare each used block under `uses`
- keep the flow serial
- bind every required input explicitly
- run `moc bcl check` before `plan` or `emit`

### Update BCL After Runtime Descriptor Changes

If `emit --check-against` fails:

1. inspect the emitted YAML
2. compare it with the checked-in `moc.yaml`
3. decide which file represents the intended structure
4. update the stale side explicitly

Do not silently bypass parity drift.

## Troubleshooting

### Problem: `moc bcl check` fails immediately

**Likely cause**: syntax or top-level statement issues.  
**Solution**: fix the `moc.bcl` source before using `plan` or `emit`.

### Problem: `plan` fails after `check` passes

**Likely cause**: flow-specific planning logic still fails under the lowered manifest.  
**Solution**: treat it as a blocking authoring error and fix the flow before emission.

### Problem: you want BCL to replace `moc.yaml`

**Likely cause**: the runtime boundary is being widened beyond current scope.  
**Solution**: keep `moc.yaml` as runtime authority and use BCL only for authoring/validation/parity.

## Best Practices

- run `init` once, then keep the source deterministic with `fmt`
- use `check` before `plan`
- use `plan` before `emit`
- use `emit --check-against` whenever the runtime descriptor should stay aligned
- keep BCL within the current MVP boundary; do not introduce unsupported constructs

## Related Topics

- [bcl_mvp_workflow.md](./bcl_mvp_workflow.md)
- [build_moc_baseline.md](./build_moc_baseline.md)
- [conformance_workflow.md](./conformance_workflow.md)
- [BCL_MOC_MVP_SPEC.md](../specs/BCL_MOC_MVP_SPEC.md)
- [BLOCKS_BCL_TOOLCHAIN_PLAN.md](../prds/BLOCKS_BCL_TOOLCHAIN_PLAN.md)
