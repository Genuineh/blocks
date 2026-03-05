---
status: draft
last_verified_commit: N/A
owner: Developer
created: 2026-03-05
updated: 2026-03-05
version: 1.0
related_prds:
  - docs/prds/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md
---

# Block Debuggability And Observability Foundation Specification

## Overview

This spec defines the minimum debuggability and observability contract that every block must satisfy during development. The goal is to make block behavior diagnosable, machine-readable, and reproducible for AI-driven build/test/verify workflows.

## Goals

- make every block execution traceable with stable identifiers
- provide consistent structured diagnostics across block implementations
- provide minimal metrics and artifacts that support fast failure reproduction
- allow `moc` workflows to correlate block events without hidden coupling

## Non-Goals

- production-grade distributed tracing backend integration
- centralized log storage design
- replacing existing block business contracts

## Scope

- `in scope active blocks` are blocks discovered by `blocks-registry` with `status: active`.
- scope snapshot is pinned to the merge commit/date of the R9 rollout phase being verified.
- archived/deprecated/experimental blocks are out of scope unless explicitly listed in TODO phase tasks.

## Architecture

### Components

- `block.yaml` extension: debug/observe capability declaration
- runtime observability wrapper: normalized execution metadata emitted at one mandatory execution boundary
- diagnostic artifact writer: local, deterministic export for failed or debug-mode executions
- CLI inspection surfaces: read/filter/export diagnostics
- moc correlation bridge: `trace_id` propagation across block executions in one run

### Data Flow

1. Caller triggers block execution (direct block call or inside `moc run`)
2. Runtime allocates `execution_id`; if inside moc run, associates or creates `trace_id`
3. Block runs and emits structured events (`start`, `success` or `failure`)
4. Runtime records metrics and writes optional artifacts by policy
5. CLI can inspect or export diagnostics for test/review/verification

Execution boundary rule:

- any execution that claims R9 compliance MUST pass through one observable runtime wrapper boundary.
- direct crate invocation from Rust `moc` is allowed only when it uses the same wrapper API and emits the same envelope.

## Contract Specification

### `block.yaml` extension (draft shape)

```yaml
debug:
  enabled_in_dev: true
  emits_structured_logs: true
  log_fields:
    - timestamp
    - level
    - event
    - block_id
    - block_version
    - execution_id
    - trace_id
observe:
  metrics:
    - execution_total
    - execution_failed_total
    - execution_latency_ms
  emits_failure_artifact: true
  artifact_policy:
    mode: on_failure # always | on_failure | never
    on_failure_minimum:
      include_input_snapshot: true
      include_error_report: true
      include_output_snapshot: conditional_when_present
    redaction_profile: basic
    retention:
      root: .blocks/diagnostics
      ttl_days: 7
      max_total_mb: 256
errors:
  taxonomy:
    - id: invalid_input
    - id: dependency_unavailable
    - id: timeout
    - id: internal_error
```

### Diagnostic event envelope

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| timestamp | string | yes | RFC3339 timestamp |
| level | string | yes | `DEBUG/INFO/WARN/ERROR` |
| event | string | yes | event name, e.g. `block.execution.start` |
| block_id | string | yes | block identifier |
| block_version | string | yes | declared version |
| execution_id | string | yes | unique per block execution |
| trace_id | string | conditional | required for `moc run`; optional for single block execution |
| duration_ms | number | no | populated on finish events |
| error_id | string | conditional | required on failure events; must match declared taxonomy |
| message | string | no | short diagnosis message |

### Failure artifact minimum payload

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| execution_id | string | yes | ties artifact to events |
| trace_id | string | no | moc-level linkage |
| block_id | string | yes | failing block |
| input_snapshot | object | yes | masked input snapshot |
| output_snapshot | object | conditional | partial output if present |
| error | object | yes | error taxonomy + detail |
| environment | object | yes | runtime mode and implementation kind |

Taxonomy constraints:

- `errors.taxonomy[].id` MUST match `^[a-z][a-z0-9_]{2,63}$`
- ids MUST be unique per block
- failure event `error_id` MUST be one of declared taxonomy ids

## CLI Surface (proposed)

Compatibility strategy:

- keep existing CLI shape (`blocks-root` argument for moc commands) and extend it
- no breaking rename in R9; add diagnose subcommands first, deprecate later only with explicit timeline

Proposed commands:

- `blocks block diagnose <block-id> [--latest|--execution-id <id>] [--json]`
- `blocks block diagnose export <block-id> --out <dir>`
- `blocks moc diagnose <blocks-root> <moc.yaml> [--trace-id <id>] [--json]`
- optional alias after rollout: `blocks diagnose ...` (deferred)

CLI behavior:

- default output should be concise human-readable summary
- `--json` must emit machine-readable output for AI agents
- non-zero exit only for command failures, not for “diagnostics contain errors”

## Technical Decisions

| Decision | Choice | Rationale |
|---------|--------|-----------|
| Correlation key | `execution_id` per block + mandatory `trace_id` per moc run | guarantees multi-block chain correlation |
| Event format | structured key-value envelope | stable for AI parsing and automation |
| Artifact policy | configurable with safe default `on_failure` | balances observability and storage costs |
| Error model | taxonomy ids in addition to free-text | supports automated remediation strategies |
| Storage policy | local `.blocks/diagnostics` + bounded retention | avoids repo pollution and unbounded growth |

## Security Considerations

- input/output snapshots must support masking and redaction profiles
- secrets/tokens/password-like keys must never be written in clear text artifacts
- diagnostic files should be local by default and excluded from accidental publication paths

`basic` redaction profile (required minimum):

- mask keys matching `(?i)(password|token|secret|authorization|api[_-]?key)`
- mask bearer token values matching `^Bearer\\s+`
- keep shape and field name, redact value

## Performance Requirements

- diagnostics overhead in default dev mode should stay lightweight (`p95` overhead ratio <= `1.10`)
- artifact writing should be bounded and configurable

Benchmark definition:

- environment: fixed CI profile
- workloads:
  - small IO block x 1000
  - pure compute block x 1000

## Testing Strategy

- contract parser tests for new `block.yaml` fields (`debug`, `observe`, `errors.taxonomy`)
- runtime tests for envelope completeness and ID correlation
- CLI tests for diagnose summary and `--json` output shape
- failure path tests proving artifact generation + redaction behavior
- moc integration tests proving `trace_id` propagation across at least two blocks
- protocol-edge tests for moc-to-moc calls (caller moc, callee moc, protocol channel, failure classification)

## Protocol-Edge Diagnostics

For `moc`-to-`moc` protocol calls, runtime diagnostics MUST include:

- `caller_moc_id`
- `callee_moc_id`
- `protocol_name`
- `channel`
- `request_summary`
- `response_summary` (or failure summary)
- `execution_id` and `trace_id`

## Verification Boundary

- automated:
  - schema validation for required debug/observe declarations
  - deterministic tests for event envelope and diagnostics export
  - regression tests for redaction and artifact policy
  - command-level checks:
    - `cargo run -p blocks-cli -- --help` includes diagnose surface after rollout
    - `cargo run -p blocks-cli -- block diagnose <block-id> --json` returns machine-readable envelope
    - `cargo run -p blocks-cli -- moc diagnose <blocks-root> <moc.yaml> --json` supports `trace_id` chain view
- manual:
  - inspect representative diagnostic outputs for readability
  - validate that failure artifacts are sufficient for local reproduction of at least one real bug case
  - confirm artifact retention and cleanup behavior under `.blocks/diagnostics`

---

### Change Log
- 2026-03-05: Added initial technical specification for mandatory block debuggability and observability foundations.
- 2026-03-05: Revised execution boundary, scope measurability, CLI compatibility, and verification contract after strict review findings.
