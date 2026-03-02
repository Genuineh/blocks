# Guide

This directory contains usage guides and contributor workflows.

Current local commands:

- `cargo test`: run the current workspace test suite.
- `cargo run -p blocks-cli -- list blocks`: list local blocks.
- `cargo run -p blocks-cli -- show blocks demo.echo`: inspect a block contract and resolved implementation path.
- `cargo run -p blocks-cli -- run blocks demo.echo /tmp/input.json`: run a single block.
- `cargo run -p blocks-cli -- compose validate blocks apps/echo-pipeline/app.yaml`: validate an app descriptor and generate a serial execution plan.
- `cargo run --manifest-path apps/echo-pipeline/backend/Cargo.toml -- blocks apps/echo-pipeline/app.yaml apps/echo-pipeline/input.example.json`: run a sample app through its Rust backend launcher.

Planned additions:

- setup and local development guide
- block creation workflow
- app composition workflow
- release and validation checklists
