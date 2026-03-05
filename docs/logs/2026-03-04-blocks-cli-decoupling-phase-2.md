---
name: Blocks CLI Decoupling Phase 2
description: Stabilize the catalog runner contract and move registration-focused coverage out of the CLI.
status: completed
created: 2026-03-04
author: Developer
---

## Development Log

### Blocks CLI Decoupling Phase 2

**Status**: Completed

**Started**: 2026-03-04 13:10
**Updated**: 2026-03-04 13:25
**Owner**: Developer

#### Objective
Implement Phase 2 by hiding the concrete catalog runner behind a stable constructor API, limiting CLI knowledge to the `BlockRunner` trait boundary in the two approved execution paths, and moving registration-focused smoke coverage into the catalog crate.

#### Progress
- [x] Hid `CatalogBlockRunner` and exposed `default_block_runner() -> impl BlockRunner`.
- [x] Rewired `blocks run` and the `moc verify` helper flow to accept `&impl BlockRunner` without changing `blocks-runtime`.
- [x] Moved the `core.http.get` registration smoke to `blocks-runner-catalog` and kept it exercising `Runtime::execute` plus real contract validation.
- [x] Reduced CLI test ownership to command/orchestration coverage and kept one `blocks run` smoke.
- [x] Updated TODO/PRD/log documentation and ran targeted cargo tests.

#### Blockers
- None.

#### Records
| Date | Action | Description |
|------|--------|-------------|
| 2026-03-04 | Created | Initialized the Phase 2 development log. |
| 2026-03-04 | Updated | Hid the concrete catalog runner behind `default_block_runner()` and rewired the two CLI execution paths to use the trait boundary. |
| 2026-03-04 | Updated | Moved unknown-block and `core.http.get` registration-focused coverage into `blocks-runner-catalog`, keeping the HTTP smoke on `Runtime::execute`. |
| 2026-03-04 | Completed | Ran `cargo test -p blocks-runner-catalog -p blocks-cli` successfully and marked Phase 2 complete in the tracking docs. |
