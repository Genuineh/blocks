---
status: active
owner: Developer
created: 2026-03-13
updated: 2026-03-13
audience: developers
level: intermediate
---

# Blocks And BCL Toolchain Handbook

## Overview

Use this guide when you want one repository-owned document that explains the full `block` / `moc` / BCL toolchain from start to finish.

This is the recommended starting point for humans and AI agents. It is CLI-first, does not depend on IDE/LSP support, and keeps one rule fixed throughout the workflow:

- `block.yaml` is the reusable capability contract
- `moc.yaml` is the runtime authority for a delivery unit
- `moc.bcl` is an authoring and validation assist, not a runtime replacement

If you only remember one loop, remember this:

1. discover existing capabilities
2. scaffold and define a `block`
3. scaffold and define a `moc`
4. add or refine `moc.bcl`
5. run evidence and conformance
6. diagnose failures
7. compare or upgrade before merging
8. run repository gate locally or in CI

## Prerequisites

- a repository layout with `blocks/` and `mocs/`
- `cargo` for the `blocks-cli` binary
- `node` only when you run frontend state tests or hosted frontend probes
- checked-in descriptors you want to author or inspect

## The Toolchain At A Glance

| Target | Authoring | Validation | Repair / Insight | Compatibility / Migration |
|--------|-----------|------------|------------------|---------------------------|
| `block` | `block init`, `block fmt` | `block check`, `block test`, `block eval`, `conformance run block` | `block doctor`, `block diagnose` | `compat block`, `upgrade block` |
| `moc` | `moc init`, `moc fmt` | `moc check`, `moc verify`, `conformance run moc` | `moc doctor`, `moc diagnose` | `compat moc`, `upgrade moc` |
| `moc.bcl` | `moc bcl init`, `moc bcl fmt` | `moc bcl check`, `moc bcl plan`, `moc bcl emit`, `conformance run bcl` | `moc bcl graph`, `moc bcl explain` | `compat bcl`, `upgrade bcl` |

## Step 1: Discover Existing Capabilities First

Before you create anything new, search the current local capability set.

Export the current catalog:

```bash
cargo run -p blocks-cli -- catalog export blocks --json
```

Search by keyword:

```bash
cargo run -p blocks-cli -- catalog search blocks echo --json
```

Inspect one block in detail:

```bash
cargo run -p blocks-cli -- show blocks demo.echo
```

Use this step to answer:

- does the needed capability already exist?
- does a nearby block exist that should be extended instead?
- is the missing work really a new `block`, or only `moc` glue?

## Step 2: Author A Reusable Block

Scaffold a new block baseline:

```bash
cargo run -p blocks-cli -- block init blocks demo.slugify --kind rust --target shared
```

Canonicalize the descriptor:

```bash
cargo run -p blocks-cli -- block fmt blocks/demo.slugify
```

Run the contract check:

```bash
cargo run -p blocks-cli -- block check blocks/demo.slugify --json
```

Then fill the repository-owned evidence folders:

- `tests/`
- `examples/`
- `evaluators/`
- `fixtures/`

Run them directly:

```bash
cargo run -p blocks-cli -- block test blocks/demo.slugify --json
cargo run -p blocks-cli -- block eval blocks/demo.slugify --json
```

Use a `block` when the capability should stay reusable across multiple `moc`s. Do not push delivery-unit wiring or product-specific orchestration into a reusable block.

Deep dive:

- [block_authoring_baseline.md](./block_authoring_baseline.md)

## Step 3: Author A Delivery MOC

Scaffold a `moc`:

```bash
cargo run -p blocks-cli -- moc init mocs/hello-service --type backend_app --backend-mode service --language rust
```

Equivalent two-argument form:

```bash
cargo run -p blocks-cli -- moc init mocs hello-service --type backend_app --backend-mode service --language rust
```

Canonicalize the descriptor:

```bash
cargo run -p blocks-cli -- moc fmt mocs/hello-service
```

Run the stable check:

```bash
cargo run -p blocks-cli -- moc check blocks mocs/hello-service --json
```

When the `moc` declares a validation flow, run it explicitly:

```bash
cargo run -p blocks-cli -- moc verify blocks mocs/echo-pipeline/moc.yaml mocs/echo-pipeline/input.example.json
```

Remember the authority boundary:

- `moc.yaml` is the delivery contract and runtime authority
- `moc run` / `moc verify` operate on `moc.yaml`, not directly on `moc.bcl`

Deep dive:

- [build_moc_baseline.md](./build_moc_baseline.md)

## Step 4: Use BCL To Assist The MOC

Scaffold BCL from the checked-in `moc.yaml`:

```bash
cargo run -p blocks-cli -- moc bcl init mocs/hello-service
```

Format and validate:

```bash
cargo run -p blocks-cli -- moc bcl fmt mocs/hello-service
cargo run -p blocks-cli -- moc bcl check blocks mocs/hello-service --json
```

Inspect the lowered plan:

```bash
cargo run -p blocks-cli -- moc bcl plan blocks mocs/hello-service/moc.bcl --json
```

Emit and compare against runtime authority:

```bash
cargo run -p blocks-cli -- moc bcl emit \
  blocks \
  mocs/hello-service/moc.bcl \
  --out .tmp/hello-service.generated.yaml \
  --check-against mocs/hello-service/moc.yaml
```

Use BCL to keep authoring structured, reviewable, and machine-checkable. Do not treat emitted YAML as a new runtime source of truth.

Deep dives:

- [bcl_authoring_baseline.md](./bcl_authoring_baseline.md)
- [bcl_mvp_workflow.md](./bcl_mvp_workflow.md)

## Step 5: Prove Conformance

Run the public conformance entrypoints:

```bash
cargo run -p blocks-cli -- conformance run block blocks/demo.echo --json
cargo run -p blocks-cli -- conformance run moc blocks mocs/echo-pipeline --json
cargo run -p blocks-cli -- conformance run bcl \
  blocks \
  mocs/echo-pipeline \
  --check-against mocs/echo-pipeline/moc.yaml \
  --gate-mode warn \
  --json
```

Use these when you want a deterministic yes/no answer for:

- is the block evidence complete?
- does the `moc` still validate as a delivery unit?
- does `moc.bcl` still stay in parity with `moc.yaml`?

Deep dive:

- [conformance_workflow.md](./conformance_workflow.md)

## Step 6: Diagnose And Repair

When a block looks incomplete or unhealthy:

```bash
cargo run -p blocks-cli -- block doctor blocks blocks/demo.echo --json
```

When a `moc` fails or drifts:

```bash
cargo run -p blocks-cli -- moc doctor blocks mocs/echo-pipeline --json
cargo run -p blocks-cli -- moc diagnose blocks mocs/echo-pipeline/moc.yaml --json
```

When BCL needs structural explanation:

```bash
cargo run -p blocks-cli -- moc bcl graph blocks mocs/echo-pipeline --json
cargo run -p blocks-cli -- moc bcl explain blocks mocs/echo-pipeline --json
```

Recommended rule:

- use `doctor` first when you want next-action guidance
- use `diagnose` when you need the latest runtime trace or artifact
- use `graph` / `explain` when the problem is in BCL authoring or parity

Deep dive:

- [discovery_diagnostics_migration_workflow.md](./discovery_diagnostics_migration_workflow.md)

## Step 7: Compare And Upgrade Before Merging

Compare descriptor compatibility:

```bash
cargo run -p blocks-cli -- compat block before/block.yaml after/block.yaml --json
cargo run -p blocks-cli -- compat moc before/moc.yaml after/moc.yaml --json
cargo run -p blocks-cli -- compat bcl blocks before/moc.bcl after/moc.bcl --json
```

Preview upgrades to the current repository-owned baseline:

```bash
cargo run -p blocks-cli -- upgrade block blocks/demo.echo --json
cargo run -p blocks-cli -- upgrade moc mocs/echo-pipeline --json
cargo run -p blocks-cli -- upgrade bcl mocs/echo-pipeline --json
```

Apply only when you want a repository-owned rewrite:

```bash
cargo run -p blocks-cli -- upgrade block blocks/demo.echo --write
```

Use `compat` before reviews. Use `upgrade` when you need canonical formatting or missing baseline directories brought back into line.

## Step 8: Run The Repository Gate

Run the full repository-owned baseline locally:

```bash
./scripts/repo_check.sh
```

Rollback the BCL parity gate temporarily without changing runtime authority:

```bash
BLOCKS_BCL_GATE_MODE=off ./scripts/repo_check.sh
```

Hosted baseline:

- [`.github/workflows/repo-check.yml`](../../.github/workflows/repo-check.yml)

The workflow reuses `./scripts/repo_check.sh` so the local and CI paths stay aligned. In CI, frontend host dependencies are prefetched before the offline Tauri probes.

## Recommended Daily Loop

For a new capability:

1. `catalog search`
2. `block init -> fmt -> check`
3. implement evidence
4. `block test -> block eval -> conformance run block`

For a new delivery unit:

1. `moc init -> fmt -> check`
2. wire blocks and protocols
3. `moc verify` when validation flow exists
4. `conformance run moc`

For BCL-assisted authoring:

1. `moc bcl init -> fmt -> check`
2. `plan`
3. `emit --check-against`
4. `conformance run bcl`

Before merging:

1. `doctor` on the affected target
2. `compat`
3. optional `upgrade --json`
4. `./scripts/repo_check.sh`

## Troubleshooting

### Problem: I do not know whether to create a block or only update a moc

**Rule**: if the capability should be reused across delivery units, create a `block`. If the work is only product-specific wiring, keep it in the `moc`.

### Problem: I have both `moc.yaml` and `moc.bcl`; which one is authoritative

**Rule**: `moc.yaml` is authoritative at runtime. `moc.bcl` helps author, validate, explain, and emit equivalent structure.

### Problem: BCL parity blocks CI temporarily

**Solution**: use `BLOCKS_BCL_GATE_MODE=off` or `warn` while repairing parity drift, then restore the stricter mode.

### Problem: The failure is runtime-shaped, not authoring-shaped

**Solution**: use `moc diagnose` or `block diagnose` instead of only `doctor`, because you need the actual trace and artifact chain.

## Best Practices

- start with `catalog search` before creating new blocks
- keep block evidence local, checked in, and deterministic
- use `doctor` before manual debugging sessions
- keep `moc.yaml` reviewable and treat BCL as assistive, not authoritative
- run `compat` before descriptor reviews and `upgrade` only when you want a baseline rewrite
- keep CI and local verification on the same `repo_check.sh` path

## Related Topics

- [block_authoring_baseline.md](./block_authoring_baseline.md)
- [build_moc_baseline.md](./build_moc_baseline.md)
- [bcl_authoring_baseline.md](./bcl_authoring_baseline.md)
- [conformance_workflow.md](./conformance_workflow.md)
- [discovery_diagnostics_migration_workflow.md](./discovery_diagnostics_migration_workflow.md)
- [bcl_mvp_workflow.md](./bcl_mvp_workflow.md)
- [BLOCKS_BCL_TOOLCHAIN_SPEC.md](../specs/BLOCKS_BCL_TOOLCHAIN_SPEC.md)
