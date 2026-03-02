# Guide

This directory contains usage guides and contributor workflows.

Current local commands:

- `cargo test`: run the current workspace test suite.
- `cargo run -p blocks-cli -- list blocks`: list local blocks.
- `cargo run -p blocks-cli -- show blocks demo.echo`: inspect a block contract path.
- `cargo run -p blocks-cli -- run blocks demo.echo /tmp/input.json`: run a single block.
- `cargo run -p blocks-cli -- compose run blocks apps/echo-pipeline/app.yaml apps/echo-pipeline/input.example.json`: run a serial app manifest.

Planned additions:

- setup and local development guide
- block creation workflow
- app composition workflow
- release and validation checklists
