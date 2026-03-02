# Repository Guidelines

## Project Structure

This repository is documentation-first.

Required documentation layout:

- `docs/TODO.md`: current work and priority order.
- `docs/prds/`: plans, architecture, and design details.
- `docs/guide/`: usage guides and contributor workflows.
- `docs/specs/`: formal specs, contracts, and repository rules.
- `docs/whitepapers/`: long-form rationale and vision.
- `docs/decisions/`: durable architecture decisions when tradeoffs matter.
- `README.md`: top-level index and project entry point.
- `CHANGELOG.md`: notable milestone or release changes after implementation starts shipping.
- `LICENSE`: must stay at the repository root.

Current content under `docs/design/` should be treated as temporary `prds` material until the tree is normalized.

Planned code layout:

- `crates/`: Rust workspace crates.
- `blocks/`: reusable blocks.
- `apps/`: composed applications.
- `skills/`: AI operating guides.

If any document moves, add or update links in the same change, including `README.md`.

## Development Workflow

Use these checks before and after edits:

- `sed -n '1,120p' docs/TODO.md`: review current priorities.
- `rg --files docs`: inspect tracked documentation.
- `git diff -- README.md docs/`: verify doc moves and link updates.

Once the Rust workspace exists, use:

- `cargo fmt --all`
- `cargo build`
- `cargo test`

## Style Rules

- Keep docs short, direct, and actionable.
- Use stable filenames and predictable structure.
- For Rust, use `rustfmt`, 4-space indentation, `snake_case` for functions/modules, and `PascalCase` for types.
- Prefer explicit names such as `BlockContract`, `ExecutionResult`, and other contract-shaped identifiers.

## Design Principles

- Start with the smallest viable engineering slice, then iterate upward.
- Small scope is not an excuse for weak design. Every iteration must preserve or improve architecture.
- Do architecture analysis before coding: affected modules, shared type ownership, dependency direction, and failure/recovery paths.
- Prefer simpler structure over extra abstraction. Reduce special cases, keep contracts consistent, and keep the runtime thin.
- Make incremental changes that stay safe under existing tests.

## Testing Rules

- Testing is a core part of development, not a final cleanup step.
- Default workflow is red/green TDD:
  1. Write a failing test.
  2. Implement the smallest change to pass it.
  3. Refactor without changing behavior.
- New behavior and bug fixes should normally start with a failing test.
- Use tests as safety rails for iteration.
- When code exists, keep unit tests near source and integration tests in each crate’s `tests/` directory.
- Name tests by behavior, for example `validates_required_inputs`.

## Acceptance Standards

A change is not done unless it meets this baseline:

- Functionality is complete, with no known omissions or obvious errors.
- Tests cover the changed behavior, key failure cases, and regression-prone paths.
- Required documentation is complete, current, and usable by the next contributor.
- Existing behavior is rechecked; no avoidable regressions are introduced.
- Naming, contracts, boundaries, and architecture remain consistent.
- The result is a maintainable solution, not a temporary patch.

If quality is uncertain, raise the bar instead of relaxing acceptance.

## Commits and Pull Requests

- Use short, imperative, capitalized commit subjects, for example `Add blocks language whitepaper`.
- Keep each commit focused on one logical change.
- PRs should include: summary, affected paths, required doc updates, and follow-up work if anything is intentionally deferred.
- Include screenshots for UI changes.
- If priorities change, update `docs/TODO.md` in the same PR.
