# Create Block

Use this workflow when a missing capability should become a reusable `block`, not a one-off patch.

## When To Create A Block

Create a new block only if all of these are true:

- the capability is reusable across more than one app or workflow
- the input and output can be described as a stable contract
- success and failure can be checked quickly
- the behavior can be kept small and single-purpose

If the work is app-specific glue, keep it in the app launcher instead of creating a block.

## Current Ground Rules

- Keep the block minimal and contract-first.
- Treat `block.yaml` as a descriptor only.
- Prefer explicit structured input and output.
- Add tests before or with the implementation.
- The actual capability must live in code:
  - `Rust` for backend or shared library blocks
  - `Tauri + TS` for frontend-only blocks
- If the block needs runtime support today, wire it into `crates/blocks-core` or the future frontend launcher.

## Workflow

1. Review available blocks first:
   `cargo run -p blocks-cli -- list blocks`
2. Create a new folder:
   `blocks/<block-id>/`
3. Add `block.yaml` with:
   - `id`
   - `name`
   - `version`
   - `status`
   - `purpose`
   - `implementation.kind`
   - `implementation.entry`
   - `implementation.target`
   - `input_schema`
   - `output_schema`
4. Add `README.md` with the block purpose and the expected result.
5. Add the actual implementation in code:
   - Rust block: `blocks/<block-id>/rust/lib.rs`
   - Frontend block: `blocks/<block-id>/tauri_ts/`
6. If the block must execute in the current MVP baseline, register it in:
   `crates/blocks-core/src/lib.rs`
7. Add or extend tests that prove:
   - invalid input fails
   - valid input succeeds
   - invalid output is rejected if relevant
8. Run:
   - `cargo fmt --all`
   - `cargo test`
9. Verify discovery:
   `cargo run -p blocks-cli -- show blocks <block-id>`

## Acceptance Checklist

- The block has one clear responsibility.
- `block.yaml` does not contain implementation logic.
- Required fields are explicit in `block.yaml`.
- Input and output schemas are both present.
- The implementation type matches the target (`rust` for backend/shared, `tauri_ts` for frontend).
- The implementation does not move contract or registry logic into the CLI or app launchers.
- Current MVP Rust blocks are linked through `blocks-core`, not re-implemented inside the CLI.
- Tests cover at least one failure path and one success path.
