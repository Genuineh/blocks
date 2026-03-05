# Guide

This directory contains usage guides and contributor workflows.

Important:

- The current sample set includes both descriptor-only `moc` files and optional validation-flow `moc` files.
- The repository-level target model is now `moc`; see `docs/specs/MOC_SPEC.md` and `docs/TODO.md`.

Current local commands:

- `./scripts/repo_check.sh`: run the current repository verification path (root workspace tests plus the standalone `counter-panel-web` Tauri host probe).
- `cargo test`: run the current root workspace test suite only.
- `cargo run -p blocks-cli -- list blocks`: list local blocks.
- `cargo run -p blocks-cli -- show blocks demo.echo`: inspect a block contract and resolved implementation path.
- `cargo run -p blocks-cli -- run blocks demo.echo /tmp/input.json`: run a single block.
- `cargo run -p blocks-cli -- block diagnose blocks demo.echo --json`: inspect the latest block diagnostic envelope and artifact in machine-readable JSON.
- `cargo run -p blocks-cli -- moc validate blocks mocs/echo-pipeline/moc.yaml`: validate the current moc descriptor, check local `internal_blocks/` layout, and, when configured, generate a serial validation plan.
- `cargo run -p blocks-cli -- moc verify blocks mocs/echo-pipeline/moc.yaml mocs/echo-pipeline/input.example.json`: explicitly run a validation flow. `moc verify` is now the only CLI path that executes `verification.entry_flow`, and it reports direct hints for bind, type, and reference mistakes.
- `cargo run -p blocks-cli -- moc diagnose blocks mocs/echo-pipeline/moc.yaml --json`: inspect the latest moc diagnostic trace chain and associated failure artifacts.
- `cargo run -p blocks-cli -- moc run blocks mocs/hello-pipeline/moc.yaml`: run a moc through the unified runner. It now prefers the real Rust backend launcher when available.
- `cargo run --manifest-path mocs/hello-pipeline/backend/Cargo.toml`: run the direct Rust-crate version of `hello-pipeline` with its default sample input.
- `cargo run --manifest-path mocs/echo-pipeline/backend/Cargo.toml`: run the direct Rust-crate version of `echo-pipeline` with its default sample input.
- `cargo run -p blocks-cli -- moc validate blocks mocs/hello-world-console/moc.yaml`: validate a descriptor-only moc.
- `cargo run -p blocks-cli -- moc run blocks mocs/hello-world-console/moc.yaml`: run a descriptor-only moc through the unified runner, which dispatches to the real backend launcher.
- `cargo run --manifest-path mocs/hello-world-console/backend/Cargo.toml`: run a zero-input backend sample that directly depends on a Rust block crate from `main`.
- `cargo run -p blocks-cli -- moc validate blocks mocs/hello-message-lib/moc.yaml`: validate a `rust_lib` moc descriptor.
- `cargo run -p blocks-cli -- moc dev blocks mocs/hello-message-lib/moc.yaml`: run the local development path for a `rust_lib` moc (currently `cargo test` on its crate).
- `cargo run -p blocks-cli -- moc validate blocks mocs/counter-panel-web/moc.yaml`: validate the interactive `frontend_app` counter example.
- `cargo run -p blocks-cli -- moc dev blocks mocs/counter-panel-web/moc.yaml`: print the human-facing web preview path, a local HTTP preview command and URL, plus the Linux app commands for the counter `frontend_app`.
- `cargo run -p blocks-cli -- moc run blocks mocs/counter-panel-web/moc.yaml`: run the counter `frontend_app` through its real Tauri host using a headless probe.
- `cargo --offline run --manifest-path mocs/counter-panel-web/src-tauri/Cargo.toml -- --headless-probe`: probe the Tauri host without opening a window.
- `cargo run --manifest-path mocs/counter-panel-web/src-tauri/Cargo.toml`: launch the real Tauri window directly.
- `cargo run -p blocks-cli -- moc validate blocks mocs/hello-panel-lib/moc.yaml`: validate the minimal `frontend_lib` moc example.
- `cargo run -p blocks-cli -- moc dev blocks mocs/hello-panel-lib/moc.yaml`: resolve the local development preview for the `frontend_lib` example and print a browser-friendly local HTTP preview command.
- `cargo run -p blocks-cli -- moc validate blocks mocs/hello-panel-web/moc.yaml`: validate the minimal `frontend_app` moc example.
- `cargo run -p blocks-cli -- moc dev blocks mocs/hello-panel-web/moc.yaml`: print the browser preview path plus a local HTTP preview command for the minimal `frontend_app` example.
- `cargo run -p blocks-cli -- moc run blocks mocs/hello-panel-web/moc.yaml`: resolve the local preview entry for the hello-panel `frontend_app`.

Planned additions:

- setup and local development guide
- block creation workflow
- moc build workflow
- release and validation checklists
