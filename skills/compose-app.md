# Compose App

Use this workflow when building an app from existing blocks with `app.yaml`.

## Current Composer Limits

The current MVP composer supports only:

- one entry flow
- serial `steps`
- `binds` using `input.<field>` or `<step-id>.<field>`
- type checks based on contract schemas
- final output equal to the last step output

Do not design branching, loops, or parallel execution yet.

Important: the current composer is a validation and transition layer, not the final app runtime model.

- `block` should be treated like a library.
- Real app behavior should live in Rust launcher code and, when there is a frontend, Tauri + TS launcher code.
- `app.yaml` may describe or validate the intended composition, but it should not be the only place where app logic exists.

## Workflow

1. List current blocks:
   `cargo run -p blocks-cli -- list blocks`
2. Choose the smallest serial flow that solves the task.
3. Create:
   `apps/<app-name>/app.yaml`
4. Define:
   - `name`
   - `entry`
   - `input_schema`
   - `flows`
   - `steps`
   - `binds`
5. Every required step input must have a bind.
6. Use only compatible types between source and target fields.
7. Add:
   - `apps/<app-name>/input.example.json`
   - `apps/<app-name>/README.md`
8. Implement the real app entrypoints:
   - `apps/<app-name>/backend/src/main.rs`
   - `apps/<app-name>/frontend/` when frontend exists
9. Use `app.yaml` as a composition descriptor or validation aid.
10. Validate the descriptor:
   `cargo run -p blocks-cli -- compose validate blocks apps/<app-name>/app.yaml`
11. Run the real app through its launcher:
   `cargo run --manifest-path apps/<app-name>/backend/Cargo.toml -- blocks apps/<app-name>/app.yaml apps/<app-name>/input.example.json`

## Debug Rules

- If you see a missing bind error, add a bind for the required field.
- If you see a type mismatch error, fix the manifest or the source block contract.
- If a step cannot run, check that the block exists in `blocks/`, its implementation entry exists, and the backend launcher links the required Rust blocks.

## Acceptance Checklist

- The flow is serial and minimal.
- Each bind is explicit and traceable.
- The app launcher runs with the example input.
- The manifest uses reusable blocks instead of app-specific hidden logic.
- The real app logic is prepared to move into launcher code, not stay only in the manifest.
