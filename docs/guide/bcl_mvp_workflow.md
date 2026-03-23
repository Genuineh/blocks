# BCL MVP Workflow

This guide explains how to use the current BCL MVP in this repository.

## What BCL Is For

In the current repository, BCL is an authoring and validation assist layer for `moc`.

Use it when you want to:

- validate a `moc.bcl` source before emitting a manifest
- inspect the planned `moc` structure in machine-readable form
- emit a deterministic `moc.yaml`
- check that a trial `moc.bcl` stays in parity with an existing `moc.yaml`

Do not use it as a runtime input.

Current authority rule:

- `moc.yaml` is still the only runtime authority
- `moc.bcl` is only for `validate`, `plan`, `emit`, and `parity`

## Current MVP Scope

The current BCL MVP supports:

- one top-level `moc`
- `name`, `type`, `language`, `entry`
- `input` / `output` schema
- `uses`
- `depends_on_mocs`
- `protocols`
- `verification` commands and optional flows
- `accept`

The current BCL MVP does not support:

- runtime or deploy code generation
- product-level orchestration outside the `moc` model
- branching, loops, `guard`, or `recover`
- block version resolution or lockfiles

## Command Workflow

The normal BCL loop is:

1. validate
2. plan
3. emit
4. parity check against the current `moc.yaml`

### Validate

```bash
cargo run -p blocks-cli -- moc bcl validate blocks mocs/echo-pipeline/moc.bcl --json
```

Use this to catch syntax, semantic, and protocol errors early.

### Plan

```bash
cargo run -p blocks-cli -- moc bcl plan blocks mocs/echo-pipeline/moc.bcl --json
```

Use this to inspect the lowered `moc` summary before emission.

Typical output includes:

- `moc_id`
- `moc_type`
- `descriptor_only`
- `uses`
- `depends_on_mocs`
- `protocols`
- `verification.entry_flow`
- flow/step/bind summary when a validation flow exists

### Emit

```bash
mkdir -p .tmp
cargo run -p blocks-cli -- moc bcl emit blocks mocs/echo-pipeline/moc.bcl --out .tmp/echo-pipeline.generated.yaml
```

Use this to generate a deterministic manifest candidate.

Default behavior:

- without `--out`, YAML is printed to stdout
- with `--out`, YAML is written to the specified path

### Check Against Existing `moc.yaml`

```bash
cargo run -p blocks-cli -- moc bcl emit blocks mocs/echo-pipeline/moc.bcl --out .tmp/echo-pipeline.generated.yaml --check-against mocs/echo-pipeline/moc.yaml
```

This proves that the emitted manifest still matches the checked-in runtime descriptor after canonical normalization.

## Trial Mocs

The current trial mocs are:

- `mocs/echo-pipeline`
  - covers flow-heavy behavior: `entry_flow`, steps, binds
- `mocs/greeting-panel-web`
  - covers protocol-heavy behavior: `depends_on_mocs`, `protocols`, descriptor-only frontend `moc`

Example:

```bash
cargo run -p blocks-cli -- moc bcl validate blocks mocs/greeting-panel-web/moc.bcl --json
cargo run -p blocks-cli -- moc bcl plan blocks mocs/greeting-panel-web/moc.bcl --json
cargo run -p blocks-cli -- moc bcl emit blocks mocs/greeting-panel-web/moc.bcl --out .tmp/greeting-panel-web.generated.yaml --check-against mocs/greeting-panel-web/moc.yaml
```

## How To Read Failures

### Validate Failure

If `validate` fails, fix the `moc.bcl` source first.

Common categories:

- syntax: malformed BCL source
- semantic: bad binds, unknown blocks, missing required inputs
- protocol: mismatched or missing `depends_on_mocs` / `protocols`

### Plan Failure

If `plan` fails after `validate` passed, treat it as a regression in the planning path or a flow-specific issue that still blocks emission.

### Parity Failure

If `emit --check-against` fails:

1. inspect the emitted YAML
2. compare it with the checked-in `moc.yaml`
3. decide which descriptor is correct
4. update either:
   - `moc.bcl`, if the runtime authority is already correct
   - `moc.yaml`, if the BCL source is the intended authoring source for that trial moc

Do not change runtime behavior by bypassing parity silently.

## Safe Contributor Rules

- Do not point runtime commands at `moc.bcl`
- Do not treat `emit` output as authoritative until parity is understood
- Do not skip `validate` before `emit`
- Do not assume `error` gate mode is the default; as of 2026-03-13 the repository default remains `warn`

## Current Limitations

- repository-level BCL gate is enabled through `conformance run bcl` and `./scripts/repo_check.sh`
- parity is structurally normalized, but additional unordered-array hardening may still be added
- BCL is still MVP scope, not the full whitepaper language

## Related Documents

- [docs/specs/BCL_MOC_MVP_SPEC.md](../specs/BCL_MOC_MVP_SPEC.md)
- [docs/prds/BCL_MOC_ASSIST_PLAN.md](../prds/BCL_MOC_ASSIST_PLAN.md)
- [docs/decisions/003-bcl-assists-moc-not-runtime.md](../decisions/003-bcl-assists-moc-not-runtime.md)
- [README.md](../../README.md)
