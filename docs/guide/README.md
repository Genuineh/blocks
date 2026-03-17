# Guide

This directory contains usage guides and contributor workflows.

Current guides:

- [blocks_bcl_toolchain_handbook.md](./blocks_bcl_toolchain_handbook.md): recommended starting guide for the full CLI-first `block` / `moc` / BCL toolchain, from discovery to CI.
- [block_authoring_baseline.md](./block_authoring_baseline.md): CLI-first public workflow for authoring reusable `block`s under the R12 Phase 2 baseline.
- [build_moc_baseline.md](./build_moc_baseline.md): CLI-first public workflow for scaffolding, formatting, and checking `moc`s under the R12 Phase 2 baseline.
- [bcl_authoring_baseline.md](./bcl_authoring_baseline.md): CLI-first public workflow for scaffolding, formatting, checking, and aligning `moc.bcl` with `moc.yaml`.
- [conformance_workflow.md](./conformance_workflow.md): public Phase 3 workflow for block evidence execution, conformance entrypoints, and BCL gate rollback.
- [discovery_diagnostics_migration_workflow.md](./discovery_diagnostics_migration_workflow.md): public Phase 4 workflow for catalog discovery, doctor/graph/explain, compat, and upgrade.
- [package_registry_baseline_workflow.md](./package_registry_baseline_workflow.md): Phase 2 workflow for `blocks pkg init/resolve/fetch/publish` and the local file-registry baseline.
- [bcl_mvp_workflow.md](./bcl_mvp_workflow.md): how to use the current BCL MVP safely, including `validate -> plan -> emit -> check-against`.

Important:

- Start with [blocks_bcl_toolchain_handbook.md](./blocks_bcl_toolchain_handbook.md) if you want one coherent end-to-end workflow instead of several focused guides.
- The current sample set includes both descriptor-only `moc` files and optional validation-flow `moc` files.
- The current execution baseline is still `moc`; see `docs/specs/MOC_SPEC.md` and `docs/TODO.md` for the migration-state rules.
- The current CLI-first guides describe the migration baseline, not the full long-term platform end state. The active long-term direction is package management + Rust-native runtime platform + BCL language; see `docs/prds/BLOCKS_BCL_TOOLCHAIN_PLAN.md`.
- The R12 Phase 2 authoring baseline is CLI-first and intentionally does not depend on IDE/LSP/editor support.
- The repository-owned hosted verification baseline is [`.github/workflows/repo-check.yml`](../../.github/workflows/repo-check.yml), which reuses `./scripts/repo_check.sh`.

Current local commands:

- `cargo run -p blocks-cli -- block init blocks demo.slugify`: scaffold a public block baseline with standard descriptor, source, and evidence folders.
- `cargo run -p blocks-cli -- block fmt blocks/demo.slugify`: canonicalize `block.yaml`.
- `cargo run -p blocks-cli -- block check blocks/demo.slugify --json`: run the stable authoring-baseline block check.
- `cargo run -p blocks-cli -- moc init mocs hello-service --type backend_app --backend-mode service --language rust`: scaffold a new `moc` baseline.
- `cargo run -p blocks-cli -- moc fmt mocs/hello-service`: canonicalize `moc.yaml`.
- `cargo run -p blocks-cli -- moc check blocks mocs/hello-service --json`: run the stable authoring-baseline moc check.
- `cargo run -p blocks-cli -- bcl init mocs/hello-service`: scaffold `moc.bcl` from an existing `moc.yaml`.
- `cargo run -p blocks-cli -- bcl fmt mocs/hello-service`: canonicalize `moc.bcl`.
- `cargo run -p blocks-cli -- bcl check mocs/hello-service --json`: run the preferred top-level BCL check for legacy workspace layouts.
- `cargo run -p blocks-cli -- bcl build packages/consumer-packaged-flow --provider workspace:packages --json`: resolve block package dependencies and emit the lowered compatibility artifact from a `bcl` package root.
- `cargo run -p blocks-cli -- block test blocks/demo.echo --json`: execute block-local functional evidence.
- `cargo run -p blocks-cli -- block eval blocks/demo.echo --json`: execute block-local evaluation evidence.
- `cargo run -p blocks-cli -- runtime check blocks/demo.echo --json`: report runtime host capability and incompatibility reasons for the current block contract.
- `cargo run -p blocks-cli -- conformance run block blocks/demo.echo --json`: aggregate block check/test/eval into a deterministic conformance report.
- `cargo run -p blocks-cli -- conformance run package packages/demo-phase2 --provider file:.tmp/file-registry --json`: verify package resolution, lockfile emission, and determinism against explicit providers.
- `cargo run -p blocks-cli -- conformance run runtime blocks/demo.echo --json`: prove the same block contract executes identically on the `sync-cli` and `tokio-service` host profiles.
- `cargo run -p blocks-cli -- conformance run moc blocks mocs/echo-pipeline --json`: run the moc conformance suite.
- `cargo run -p blocks-cli -- conformance run bcl mocs/echo-pipeline --check-against mocs/echo-pipeline/moc.yaml --gate-mode warn --json`: run legacy-layout BCL conformance and parity gating.
- `cargo run -p blocks-cli -- conformance run bcl packages/consumer-packaged-flow --provider workspace:packages --gate-mode off --json`: run package-aware BCL conformance over a `bcl` package root.
- `cargo run -p blocks-cli -- catalog export blocks --json`: export the current local block catalog as stable JSON.
- `cargo run -p blocks-cli -- catalog search blocks echo --json`: search the local catalog for AI-friendly discovery.
- `cargo run -p blocks-cli -- block doctor blocks blocks/demo.echo --json`: inspect contract, evidence, and latest diagnostics for a reusable block.
- `cargo run -p blocks-cli -- moc doctor blocks mocs/echo-pipeline --json`: inspect descriptor, launcher, protocol, and trace health for a moc.
- `cargo run -p blocks-cli -- bcl graph mocs/echo-pipeline --json`: export the lowered BCL assembly graph.
- `cargo run -p blocks-cli -- bcl explain mocs/echo-pipeline --json`: get a repair-oriented BCL explanation in success or failure cases.
- `cargo run -p blocks-cli -- compat block before/block.yaml after/block.yaml --json`: classify block descriptor changes as compatible or breaking.
- `cargo run -p blocks-cli -- upgrade block blocks/demo.echo --json`: preview a baseline migration to the Phase 4 toolchain conventions.
- `BLOCKS_BCL_GATE_MODE=off ./scripts/repo_check.sh`: rollback the BCL parity gate and keep repository checks on pure `moc.yaml` authority.
- `./scripts/repo_check.sh`: run the current repository verification path (workspace tests, conformance, catalog/doctor/compat/upgrade checks, and the standalone frontend probes).
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
- release and validation checklists
