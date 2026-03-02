# Create Block

Use this workflow when a missing capability should become a reusable `block`, not a one-off patch.

## When To Create A Block

Create a new block only if all of these are true:

- the capability is reusable across more than one app or workflow
- the input and output can be described as a stable contract
- success and failure can be checked quickly
- the behavior can be kept small and single-purpose

If the work is app-specific glue, keep it in the app manifest instead of creating a block.

## Current Ground Rules

- Keep the block minimal and contract-first.
- Prefer explicit structured input and output.
- Add tests before or with the implementation.
- If the block needs runtime support today, wire it into the current `CliBlockRunner`.

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
   - `input_schema`
   - `output_schema`
4. Add `README.md` with the block purpose and the expected result.
5. If the block must execute in the current MVP, add the smallest implementation to:
   `crates/blocks-cli/src/main.rs`
6. Add or extend tests that prove:
   - invalid input fails
   - valid input succeeds
   - invalid output is rejected if relevant
7. Run:
   - `cargo fmt --all`
   - `cargo test`
8. Verify discovery:
   `cargo run -p blocks-cli -- show blocks <block-id>`

## Acceptance Checklist

- The block has one clear responsibility.
- Required fields are explicit in `block.yaml`.
- Input and output schemas are both present.
- The implementation does not move contract or registry logic into the CLI.
- Tests cover at least one failure path and one success path.

