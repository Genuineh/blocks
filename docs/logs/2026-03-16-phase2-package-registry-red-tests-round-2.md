---
name: phase2-package-registry-red-tests-round-2
description: Second RED cycle test log for Phase 2 review-gap coverage.
status: completed
created: 2026-03-16
author: Tester
---

## Development Log

### Phase 2 Package/Registry RED Tests (Round 2)

**Status**: Completed

**Started**: 2026-03-16 00:00
**Updated**: 2026-03-16 00:00
**Owner**: Tester

#### Objective
Implement supplemental RED tests for reviewer-discovered Phase 2 gaps without changing production code.

#### Approved Supplemental RED Scope
- Provider precedence fallback behavior
- Conflicting release detection across providers
- Lockfile dependency entries must store concrete resolved versions
- Fetch path must support non-default versions
- Strict/compat unknown-key behavior
- Fetch output split between JSON and human mode

#### Records
| Date | Action | Description |
|------|--------|-------------|
| 2026-03-16 | Created | Extended `crates/blocks-cli/tests/pkg_phase2_red.rs` with supplemental RED cases for review gaps. |
| 2026-03-16 | Verified | Ran `cargo test -p blocks-cli --test pkg_phase2_red`; observed `7 failed / 5 passed`, and failures map to unresolved implementation gaps in resolver/fetch/validation behavior. |

#### Notes
- This round intentionally adds failing tests to enforce reviewer findings before the next GREEN iteration.
- No production crate logic was changed in this RED cycle.
