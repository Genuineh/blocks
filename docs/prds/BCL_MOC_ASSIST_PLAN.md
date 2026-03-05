---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-05
updated: 2026-03-05
version: 1.0
---

# BCL Moc Assist Plan

## Summary

Define a practical BCL establishment plan that stays fully aligned with the current `moc` model: BCL is an authoring and validation assist layer for `moc`, not a replacement for `moc` and not a runtime/deploy code generator.

## Problem

The repository already has a BCL vision in whitepaper form, but lacks an executable PRD/Spec path. Without a constrained MVP boundary, implementation can drift into a second top-level model that conflicts with `MOC_SPEC`.

## Users

- moc authors who need lower-friction authoring with strict correctness checks
- reviewers who need deterministic, auditable planning artifacts
- AI agents that need machine-readable diagnostics and stable generation/validation workflows
- maintainers of `blocks-moc` and `blocks-cli`

## Requirements

### Must Have

- BCL MVP explicitly constrained to `moc`-assist scope:
  - parse and validate `moc.bcl`
  - produce a normalized `moc.yaml` (or parity report)
  - produce machine-readable diagnostics with stable rule IDs and source spans
- preserve `moc.yaml` as runtime authority in MVP (no runtime/deploy generation)
- CLI surface under `moc` namespace:
  - `blocks moc bcl validate`
  - `blocks moc bcl plan`
  - `blocks moc bcl emit`
- semantic checks must align with existing `blocks-moc` behavior:
  - `uses.blocks` vs flow step consistency
  - bind reference/type validity
  - cross-moc protocol compatibility
- compatibility and rollback gate:
  - BCL adoption is opt-in per moc
  - disabling BCL must keep existing `moc run/verify/diagnose` behavior unchanged

### Should Have

- deterministic normalized output for stable diff/review
- parity check against existing `moc.yaml`
- integration in `./scripts/repo_check.sh` as opt-in warn gate first

### Nice to Have

- editor-friendly diagnostics index under `.blocks/bcl-diagnostics/`
- auto-generated starter `moc.bcl` from existing `moc.yaml`

## Non-Goals

- replacing `moc` as the delivery unit
- introducing a second top-level product type model (e.g. `product/service/runtime`)
- runtime code generation, deploy packaging, or orchestration engine generation
- block version resolver/lockfile in MVP

## User Stories

- As a moc author, I want to write a constrained `moc.bcl` and get strict semantic diagnostics before runtime verification.
- As a reviewer, I want deterministic `emit` output so BCL changes are diff-friendly and auditable.
- As a maintainer, I want BCL rollout to be reversible and not break existing moc command behavior.

## Success Metrics

- 100% of BCL trial mocs pass `emit --check-against <moc.yaml>` parity checks.
- `blocks moc bcl validate --json` emits stable machine-readable diagnostics for syntax/semantic/protocol errors.
- Enabling BCL gate in warn mode introduces zero regressions to existing `moc run/verify/diagnose` command behavior.

## Timeline

### Phase 1: Boundary Freeze (docs only)
- finalize PRD/Spec/ADR
- freeze MVP grammar and semantic scope
- freeze CLI command shape and diagnostics contract

Entry gate:
- architecture/reviewer sign-off completed

Exit gate:
- active PRD + draft spec + proposed ADR merged
- TODO phase plan and acceptance checks added

### Phase 2: Validate Path
- create `blocks-bcl` parser + AST + semantic validator
- add `blocks moc bcl validate`
- emit structured diagnostics (`error_id`, `rule_id`, `span`, `hint`)

Entry gate:
- Phase 1 docs approved

Exit gate:
- syntax/semantic/protocol failures are test-covered and reproducible

### Phase 3: Plan/Emit + Parity
- add `blocks moc bcl plan --json`
- add `blocks moc bcl emit`
- add `--check-against <moc.yaml>` parity checks

Entry gate:
- validate path stable with regression tests

Exit gate:
- at least 2 trial mocs pass parity checks

### Phase 4: Migration Gate
- trial rollout on selected mocs
- repo check integration in warn mode, then conditional error mode
- rollback switch documented and tested

Entry gate:
- Phase 3 parity proven on trial mocs

Exit gate:
- BCL gate behavior and rollback path validated in repository checks

## Open Questions

- Should `moc.bcl` live next to every trial `moc.yaml` by default, or only in opted-in directories?
- Should `plan` JSON include optional derived quality hints, or strictly structural IR in MVP?

## Acceptance Criteria

- [x] A constrained BCL plan is documented and aligned to `moc` authority.
- [x] MVP phases include executable entry/exit gates and rollback expectations.
- [x] TODO contains phase tasks and acceptance criteria for implementation follow-up.

---

### Change Log
- 2026-03-05: Added BCL establishment plan constrained to moc-assist scope with phased rollout gates.
