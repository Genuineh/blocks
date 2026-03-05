# src-tauri

This directory marks the Tauri host boundary for `counter-panel-web`.

The current MVP now includes a real Tauri host crate:

- a toolchain-free preview at `../preview/index.html`
- a Tauri runtime entry at `src/main.rs`
- a safe automation path via `--headless-probe`

The normal `cargo run --manifest-path src-tauri/Cargo.toml` command launches the real Tauri application. The `--headless-probe` flag checks the embedded frontend path without opening a window, which is what `blocks moc run` uses in terminal-only workflows. In offline environments, use `cargo --offline run --manifest-path src-tauri/Cargo.toml -- --headless-probe`.
