# src-tauri

This directory marks the Tauri host boundary for `greeting-panel-web`.

The host keeps the same bounded automation pattern as the existing frontend samples:

- `cargo run --manifest-path src-tauri/Cargo.toml` launches the real desktop host
- `cargo --offline run --manifest-path src-tauri/Cargo.toml -- --headless-probe` verifies the preview assets exist without opening a window

The headless probe is a host-readiness check only. It does not prove that the browser runtime fetched and rendered live backend data.
