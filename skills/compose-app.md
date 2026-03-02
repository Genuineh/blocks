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
8. Run:
   `cargo run -p blocks-cli -- compose run blocks apps/<app-name>/app.yaml apps/<app-name>/input.example.json`

## Debug Rules

- If you see a missing bind error, add a bind for the required field.
- If you see a type mismatch error, fix the manifest or the source block contract.
- If a step cannot run, check that the block exists in `blocks/` and has a current executor.

## Acceptance Checklist

- The flow is serial and minimal.
- Each bind is explicit and traceable.
- The app runs with the example input.
- The manifest uses reusable blocks instead of app-specific hidden logic.

