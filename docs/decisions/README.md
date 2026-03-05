# Decisions

This directory stores durable architecture decisions when a tradeoff needs long-term traceability.

Recommended pattern:

- one decision per file
- include context, decision, consequences, and date

Current decisions:

- `001-enforce-contract-runtime-boundary.md`: align contract enforcement, runtime observability boundary, and CLI layering.
- `002-r10-phase1-runtime-observability-boundary.md`: minimal Phase 1 decision for run/verify boundary unification, moc_id diagnostics ownership, and taxonomy-first error mapping with controlled fallback.
