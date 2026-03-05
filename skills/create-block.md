# Create Block

Use this workflow when a missing capability should become a reusable `block`, not a one-off patch.

Current model note:

- `block` remains valid under the new architecture.
- The upper delivery unit is now `moc`.

## When To Create A Block

Create a new block only if all of these are true:

- the capability is reusable across more than one moc or workflow
- the input and output can be described as a stable contract
- success and failure can be checked quickly
- the behavior can be kept small and single-purpose

If the work is moc-specific glue, keep it in the moc entrypoint instead of creating a block.

## Current Ground Rules

- Keep the block minimal and contract-first.
- Treat `block.yaml` as a descriptor only.
- Prefer explicit structured input and output.
- Add tests before or with the implementation.
- The actual capability must live in code:
  - `Rust` for backend or shared library blocks
  - `Tauri + TS` for frontend-only blocks
- If the block needs backend CLI debug-run support today, register it by adding its `block-*` path dependency in `crates/blocks-runner-catalog/Cargo.toml`. The catalog’s build-time glue is generated from that manifest.

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
   - When the block should be directly reusable by Rust `moc`s, also add:
     `blocks/<block-id>/rust/Cargo.toml`
   - Frontend block: `blocks/<block-id>/tauri_ts/`
6. If the block must execute through the current CLI debug path, register it in:
   `crates/blocks-runner-catalog/Cargo.toml`
   using a `block-* = { path = "..." }` dependency entry that points at the block’s `rust/` crate.
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
- Reusable Rust blocks should prefer a direct crate entry, not only a registry-only path.
- The implementation does not move contract or registry logic into the CLI or moc entrypoints.
- Current MVP no longer requires `blocks-core`; reusable Rust blocks should ship and remain callable through their own crate entry.
- Tests cover at least one failure path and one success path.
