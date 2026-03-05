---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-05
updated: 2026-03-05
version: 1.0
---

# Block Debuggability And Observability Foundation Plan

## Summary

Define a repository-wide baseline so every block is debuggable and observable during development, enabling AI and humans to diagnose failures, verify behavior, and evaluate block-composed moc products with lower ambiguity.

## Problem

Current block contracts emphasize schema and verification, but do not yet enforce a unified debuggability and observability baseline. This leads to uneven diagnostics quality across blocks and raises the cost of AI-driven test/verify loops when failures occur.

## Users

- block authors
- moc authors composing multiple blocks
- AI agents running `build -> test -> verify` workflows
- reviewers validating reliability and failure boundaries

## Requirements

### Must Have

- a mandatory debuggability baseline for every block in development mode:
  - stable execution identity (`execution_id`, `block_id`, `block_version`)
  - structured logs with fixed core fields
  - explicit error taxonomy and failure metadata
- a mandatory observability baseline:
  - minimum execution metrics (success/failure count, latency, retry count when applicable)
  - deterministic artifacts for failure diagnosis (input snapshot with masking rules, output snapshot, error report)
- block contract extension in `block.yaml` for debug/observe declarations
- CLI support for block-level diagnostics export and quick inspection
- clear boundary between automated observability checks and manual diagnosis workflows

### Should Have

- replayable fixture package for failed block executions
- cross-block correlation propagation for one moc run (`trace_id` at moc boundary, linked `execution_id` per block)
- standardized severity model (`DEBUG/INFO/WARN/ERROR`) and event naming

### Nice to Have

- simple local timeline view for a moc run assembled from block diagnostic artifacts
- lightweight redaction policy templates by data class

## User Stories

- As a block author, I want every failed execution to emit consistent diagnostic artifacts, so that I can quickly reproduce and fix issues.
- As a moc author, I want correlated block diagnostics in one run, so that I can locate the failing block and dependency edge fast.
- As an AI agent, I want machine-readable logs/metrics/errors, so that I can choose better next tests and verification steps automatically.

## Success Metrics

Scope definition (normative):

- `active block` = blocks discovered by `blocks-registry` whose `block.yaml` has `status: active`.
- Scope snapshot for this plan is fixed at merge commit of this PRD version.

Metrics:

- 100% of in-scope active blocks declare required `debug`/`observe`/`errors.taxonomy` fields in `block.yaml`.
- 100% of failed in-scope block executions emit structured diagnostics containing `execution_id`.
- >= 80% of benchmark failure cases in `docs/specs/diagnostics_benchmark_cases.md` can be reproduced from emitted artifacts without reconstructing inputs manually.
- `moc` multi-block benchmark median identification time is reduced by at least 50% against baseline window `2026-03-01` to `2026-03-15`.

## Timeline

- Phase 1: spec and schema authority alignment (`BLOCKS_SPEC`, R9 scope, phase gates)
- Phase 2: CLI and contract validator support
- Phase 3: migration of explicit in-scope block set
- Phase 4: moc-level correlation, protocol-edge diagnostics, and verification hardening

Phase acceptance contract:

- each phase must define explicit entry/exit criteria in `docs/TODO.md`
- each phase must map to executable checks (command and expected artifact)

## Open Questions

- Should masked input snapshots be mandatory for all blocks or only side-effecting blocks?
- For frontend-only blocks, should observability schema be fully aligned with backend blocks or allow partial profiles?

## Acceptance Criteria

- [x] A plan exists for mandatory block debuggability and observability foundations.
- [x] The plan defines required contract, CLI, and verification boundaries.
- [x] Implementation tasks are tracked in `docs/TODO.md` with explicit phase acceptance.

---

### Change Log
- 2026-03-05: Added the baseline plan for mandatory block debuggability and observability capabilities.
- 2026-03-05: Revised scope, measurable metrics, and phase acceptance contract after strict review findings.
