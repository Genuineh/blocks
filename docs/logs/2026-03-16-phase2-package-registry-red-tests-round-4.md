---
name: phase2-package-registry-red-tests-round-4
description: Final shim-removal RED cycle for dep.sample/path-sentinel compatibility behavior.
status: completed
created: 2026-03-16
author: Tester
---

## Development Log

### Phase 2 Package/Registry RED Tests (Round 4)

**Status**: Completed

**Started**: 2026-03-16 00:00
**Updated**: 2026-03-16 00:00
**Owner**: Tester

#### Objective
Implement a strict RED cycle that fails on the remaining dep.sample compatibility shim blockers, without any production code changes.

#### Added RED Coverage
- default resolve must not synthesize `dep.sample`
- missing `dep.sample` must fail as `pkg.resolve.unsatisfied_constraint`
- `--lock` must not emit `blocks.lock` when only shim would make resolve succeed
- provider result must not depend on path sentinel naming (`empty`)
- fallback must happen only when next provider has a real release
- no false conflict when only one provider has real release
- conflict detection requires two real discovered releases
- lockfile dependency version must come from provider-discovered release
- compat shim default-off behavior
- compat shim explicit opt-in behavior (if retained)

#### Records
| Date | Action | Description |
|------|--------|-------------|
| 2026-03-16 | Created | Extended `crates/blocks-cli/tests/pkg_phase2_red.rs` with dep.sample/path-sentinel focused RED tests for shim removal. |
| 2026-03-16 | Verified | Ran `cargo test -p blocks-cli --test pkg_phase2_red` and captured intentional failures that map to unresolved shim behavior. |

#### Notes
- This round is intentionally RED and constrains only shim-removal blockers.
- No production crate logic was changed in this cycle.
