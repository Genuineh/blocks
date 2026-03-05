---
adr_number: 003
date: 2026-03-05
status: proposed
author: Developer
reviewed_by:
  - Reviewer (fus-lead, 2026-03-05)
---

# 003: BCL Assists Moc Authoring Without Replacing Runtime Authority

## Status
Proposed

## Context
The repository has a BCL whitepaper vision, while the implemented architecture has already converged on `moc` as the only delivery unit and `moc.yaml` as the runtime-facing descriptor.

Without an explicit decision boundary, BCL implementation can drift into:
- a second top-level delivery model
- runtime/deploy code generation scope
- incompatible semantics with current `blocks-moc` and `blocks-cli`

## Decision
Adopt the following boundary for BCL MVP:

1. BCL is an assist layer for `moc` authoring and validation.
2. `moc.yaml` remains runtime authority during MVP.
3. BCL CLI stays under `blocks moc bcl ...` namespace.
4. BCL MVP excludes runtime/deploy generation and version resolver/lockfile features.
5. BCL outputs must pass existing `blocks-moc` validation/parity checks before acceptance.

## Consequences

### Positive
- Preserves current architecture and avoids dual-model drift.
- Enables incremental rollout with reversible gates.
- Keeps observability and runtime boundaries unchanged.

### Negative
- Short-term dual artifact maintenance (`moc.bcl` + `moc.yaml`) for opted-in mocs.
- Some whitepaper-level ambitions are explicitly deferred.

## Alternatives Considered

### Alternative 1: BCL becomes immediate runtime authority
**Pros**: single source eventually.  
**Cons**: high migration risk, breaks current validation/runtime ownership boundaries.  
**Why Rejected**: not suitable for MVP and incompatible with current stability goals.

### Alternative 2: BCL only as lint layer, no emit/parity
**Pros**: faster initial delivery.  
**Cons**: cannot close loop on deterministic generation and reviewability.  
**Why Rejected**: weak implementation value for AI authoring workflows.

## Notes
- Related PRD: `docs/prds/BCL_MOC_ASSIST_PLAN.md`
- Related Spec: `docs/specs/BCL_MOC_MVP_SPEC.md`
- Constrained by: `docs/specs/MOC_SPEC.md`
