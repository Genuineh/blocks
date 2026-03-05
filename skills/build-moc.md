# Build MOC

Use this workflow when building a moc from existing blocks with `moc.yaml`.

## Current MOC Planner Limits

The current MVP validation planner supports only:

- one optional validation flow
- serial `steps`
- `binds` using `input.<field>` or `<step-id>.<field>`
- type checks based on contract schemas
- final output equal to the last step output

Do not design branching, loops, or parallel execution in the validation flow yet.

Important: the current moc planner is a validation and transition layer, not the final moc runtime model.

- `block` should be treated like a library.
- Real moc behavior should live in Rust launcher code and, when there is a frontend, Tauri + TS launcher code.
- `moc.yaml` may describe or validate the intended composition, but it should not be the only place where moc logic exists.

## Workflow

1. List current blocks:
   `cargo run -p blocks-cli -- list blocks`
2. Choose the smallest moc shape that solves the task.
3. Create:
   `mocs/<moc-name>/moc.yaml`
4. Define:
   - `id`
   - `name`
   - `type`
   - `backend_mode` when `type=backend_app`
   - `language`
   - `entry`
   - `public_contract`
   - `uses.blocks`
   - `uses.internal_blocks`
   - `depends_on_mocs`
   - `protocols`
   - `verification`
   - `acceptance_criteria`
5. Implement the real moc entrypoints first:
   - `mocs/<moc-name>/backend/src/main.rs`
   - `mocs/<moc-name>/frontend/` when frontend exists
6. If serial validation helps, add `verification.entry_flow` plus `verification.flows`.
7. Every required step input in the validation flow must have a bind.
8. Use only compatible types between source and target fields in the validation flow.
9. Add:
   - `mocs/<moc-name>/input.example.json`
   - `mocs/<moc-name>/README.md`
10. Use `moc.yaml` as a descriptor and validation aid, not as the only runtime.
11. Validate the descriptor:
   `cargo run -p blocks-cli -- moc validate blocks mocs/<moc-name>/moc.yaml`
12. Use the unified moc runner:
   `cargo run -p blocks-cli -- moc run blocks mocs/<moc-name>/moc.yaml`
   If the moc has a real launcher or preview path, the CLI will use that runtime path.
   `moc run` no longer executes `verification.entry_flow`.
13. When you need to execute the validation flow explicitly:
   `cargo run -p blocks-cli -- moc verify blocks mocs/<moc-name>/moc.yaml`
   The CLI now reports direct hints for missing binds, type mismatches, and invalid references.
14. Run the real moc through its launcher when needed:
   `cargo run --manifest-path mocs/<moc-name>/backend/Cargo.toml`

## Debug Rules

- If you see a missing bind error, add a bind for the required field in `verification.flows`.
- If you see a type mismatch error, fix the validation flow or the source block contract.
- If validation fails before planning, fix `type`, `backend_mode`, `protocols`, or other descriptor metadata first.
- If `moc run` refuses to execute, add a real launcher or preview path. If you only want to test the validation flow, use `moc verify`.
- If a block cannot run, check that it exists in `blocks/`, its implementation entry exists, and the backend launcher links the required Rust blocks.

## Acceptance Checklist

- The descriptor declares exactly one moc type.
- The real launcher owns behavior.
- The validation flow, when present, is serial and minimal.
- Each bind in the validation flow is explicit and traceable.
- The moc launcher runs with the example input.
- The manifest uses reusable blocks instead of moc-specific hidden logic.
- The real moc logic stays in launcher code, not only in the manifest.
