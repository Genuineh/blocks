---
name: Blocks CLI Decoupling Phase 1
description: Extract CLI-owned runnable Rust block wiring into a dedicated catalog crate.
status: completed
created: 2026-03-04
author: Developer
---

## Development Log

### Blocks CLI Decoupling Phase 1

**Status**: Completed

**Started**: 2026-03-04 13:00
**Updated**: 2026-03-04 13:05
**Owner**: Developer

#### Objective
Implement Phase 1 of the decoupling plan by moving runnable Rust block dispatch out of `blocks-cli` and into a new catalog crate while preserving behavior.

#### Progress
- [x] Identified the existing `CliBlockRunner` dispatch table and CLI call sites.
- [x] Added `crates/blocks-runner-catalog` and moved the dispatch table there unchanged.
- [x] Added catalog-owned regression tests for a known dispatch and the exact unknown-block fallback.
- [x] Run targeted cargo tests and record final status.

#### Blockers
- None.

#### Records
| Date | Action | Description |
|------|--------|-------------|
| 2026-03-04 | Created | Initialized the Phase 1 development log. |
| 2026-03-04 | Updated | Extracted runnable Rust block wiring into `blocks-runner-catalog` and rewired `blocks-cli`. |
| 2026-03-04 | Completed | Ran `cargo test -p blocks-runner-catalog -p blocks-cli` successfully and closed the implementation log. |
| 2026-03-04 | Referenced | Updated `docs/TODO.md` to track the decoupling plan by Phase and link back to this completed Phase 1 record. |
