---
status: draft
last_verified_commit: N/A
owner: Developer
created: 2026-03-05
updated: 2026-03-05
version: 1.0
related_prds:
  - docs/prds/BCL_MOC_ASSIST_PLAN.md
---

# BCL Moc MVP Specification

## Overview

This specification defines the MVP technical boundary for BCL in this repository.

Authoritative rule:
- `moc` remains the delivery unit.
- `moc.yaml` remains the runtime authority in MVP.
- `moc.bcl` is an assist source for validation and deterministic manifest generation.

## Goals

- provide a constrained BCL syntax for moc authoring
- provide deterministic semantic validation aligned with `blocks-moc`
- provide optional deterministic `moc.yaml` generation and parity checks
- provide machine-readable diagnostics suitable for AI-assisted workflows

## Non-Goals

- runtime/deploy code generation
- replacing `moc.yaml` as runtime authority in MVP
- introducing new top-level product taxonomy outside `MOC_SPEC`
- block version resolver and lockfile

## Architecture

### Components

- `crates/blocks-bcl` (new)
  - `syntax`: lexer/parser/span -> AST
  - `ir`: AST normalization -> `BclMocIr`
  - `sema`: semantic checks against registry/moc rules
  - `emit`: deterministic moc manifest emission + parity check helpers
- `crates/blocks-cli`
  - new subcommands under `moc bcl` namespace
- existing crates reused as dependencies
  - `blocks-registry` for block discovery
  - `blocks-moc` for manifest/parity validation rules
  - `blocks-contract` for schema/type compatibility checks

### Boundary Rules

- BCL logic must not enter `blocks-runtime`.
- BCL output must be validated again via `MocManifest` path before considered valid.
- BCL diagnostics storage (`.blocks/bcl-diagnostics`) must remain separate from runtime diagnostics.

## CLI Specification

### `blocks moc bcl validate`

Command:
- `blocks moc bcl validate <blocks-root> <moc.bcl> [--json] [--against <moc.yaml>]`

Behavior:
- parse + semantic checks
- optional parity check with provided manifest (mandatory in repo gate for opted-in mocs)
- no file generation

### `blocks moc bcl plan`

Command:
- `blocks moc bcl plan <blocks-root> <moc.bcl> [--json]`

Behavior:
- output normalized IR summary
- no runtime behavior

### `blocks moc bcl emit`

Command:
- `blocks moc bcl emit <blocks-root> <moc.bcl> [--out <path>] [--check-against <moc.yaml>]`

Behavior:
- generate deterministic normalized `moc.yaml`
- optional parity check in command surface, but mandatory in repository gate for opted-in mocs

## BCL -> MOC Canonical Mapping (Normative)

To avoid dual-model drift, emitted manifest keys must follow `MOC_SPEC` canonical names:

- `backend_app(console|service)` in BCL maps to:
  - `type: backend_app`
  - `backend_mode: console|service`
- BCL `uses { block ...; internal_block ...; }` maps to:
  - `uses.blocks`
  - `uses.internal_blocks`
- verification flows map to:
  - `verification.flows`
  - exactly one `verification.entry_flow` when any flow is declared
- BCL dependency/protocol declarations map directly to:
  - `depends_on_mocs`
  - `protocols`

## MVP Grammar (EBNF)

```ebnf
file         = "moc" ident "{" stmt* "}" ;
stmt         = name_stmt | type_stmt | language_stmt | entry_stmt | uses_stmt | deps_stmt | protocols_stmt | verification_stmt | acceptance_stmt ;
name_stmt    = "name" string ";" ;
type_stmt    = "type" moc_type ";" ;
moc_type     = "rust_lib" | "frontend_lib" | "frontend_app" | "backend_app" ["(" ("console"|"service") ")"] ;
language_stmt= "language" ("rust"|"tauri_ts") ";" ;
entry_stmt   = "entry" string ";" ;
uses_stmt    = "uses" "{" use_item* "}" ;
use_item     = "block" ident ";" | "internal_block" ident ";" ;
deps_stmt    = "depends_on_mocs" "{" dep_item* "}" ;
dep_item     = "moc" string "via" ident ";" ;
protocols_stmt = "protocols" "{" protocol* "}" ;
protocol     = "protocol" ident "{" "channel" ident ";" "input" schema "output" schema "}" ;
schema       = "{" field* "}" ;
field        = ident ":" scalar_type ["required"] ";" ;
scalar_type  = "string" | "number" | "integer" | "boolean" | "object" | "array" ;
verification_stmt = "verification" "{" verify_item* "}" ;
verify_item  = "command" string ";" | flow ;
flow         = ["entry"] "flow" ident "{" step+ bind* "}" ;
step         = "step" ident "=" ident ";" ;
bind         = "bind" ref "->" ref ";" ;
ref          = "input." ident | ident "." ident ;
acceptance_stmt = "accept" string ";" ;
```

## Semantic Rules

1. Top-level entity must be exactly one `moc`.
2. `type` values must match `MOC_SPEC`.
3. `backend_app` requires explicit `console|service`; other types must not set backend mode.
4. Every flow step block must be declared in BCL `uses { block ... }` and emitted to `uses.blocks`.
5. Every declared BCL `uses { block ... }` used by verification flow must be reachable in flow steps.
6. `bind` references only allow `input.<field>` and previous-step output references.
7. bind type compatibility must match block contract input/output schemas.
8. protocol dependency checks must match both sides (`name/channel/input_schema/output_schema`).
9. BCL must reject unsupported constructs (`guard`, `recover`, branching, loops) in MVP.
10. If any verification flow exists, exactly one flow must be marked as entry and emitted to `verification.entry_flow`.
11. Emitted manifest must use canonical field names from `MOC_SPEC` (`backend_mode`, `uses.blocks`, `uses.internal_blocks`, `verification.entry_flow`).

## Source Of Truth Rule (Normative)

- Runtime authority is always `moc.yaml`.
- For opted-in mocs, repository check must run parity gate:
  - `blocks moc bcl emit <blocks-root> <moc.bcl> --check-against <moc.yaml>`
- In error mode, parity mismatch must fail gate.
- `emit` should default to stdout; writing files via `--out` is allowed only for explicit generation workflows.

## Diagnostics Contract

### Output Shape (`--json`)

```json
{
  "status": "error|ok",
  "source": "path/to/moc.bcl",
  "rule_results": [
    {
      "error_id": "bcl.semantic.bind_type_mismatch",
      "rule_id": "BCL-SEMA-004",
      "severity": "error|warn",
      "message": "...",
      "hint": "...",
      "span": {
        "line": 12,
        "column": 9,
        "end_line": 12,
        "end_column": 24
      }
    }
  ]
}
```

Required fields:
- `error_id`
- `rule_id`
- `severity`
- `message`
- `span`

## Test Strategy

- parser unit tests (valid/invalid grammar)
- semantic unit tests:
  - unknown block
  - undeclared dependency usage
  - invalid bind reference
  - bind type mismatch
  - protocol mismatch
- CLI integration tests:
  - `validate --json` contract stability
  - `plan --json` deterministic output
  - `emit --check-against` parity pass/fail
- regression tests proving existing `moc run/verify/diagnose` behavior unaffected when BCL is not used

## Rollout And Gating

- Phase 2: BCL checks off by default, explicit command invocation only
- Phase 3: opt-in parity checks on selected mocs
- Phase 4: repo gate in warn mode first; error mode only after parity stability window

Rollback requirement:
- disabling BCL checks must not affect existing moc command workflows

## Acceptance

- grammar and semantic checks are deterministic and test-covered
- generated `moc.yaml` passes existing `blocks-moc` validation
- parity checks are stable for trial mocs
- no runtime boundary regression introduced
