# counter-panel-web

Minimal `frontend_app` moc example that behaves like a small counter app.

It directly imports the frontend block `ui.counter.mount` from code, while `moc.yaml` remains the descriptor used for AI guidance and validation.

Structure:

- `src/index.html`: minimal source page shell with `#app`
- `src/main.ts`: frontend entrypoint
- `preview/index.html`: toolchain-free static preview shell
- `preview/main.js`: toolchain-free preview bootstrap
- `preview/shell.css`: shared HTML shell styling reused by both source and preview
- `src-tauri/`: real Tauri host crate

Validate the descriptor:

```bash
cargo run -p blocks-cli -- moc validate blocks mocs/counter-panel-web/moc.yaml
```

Get the human-facing verification paths:

```bash
cargo run -p blocks-cli -- moc dev blocks mocs/counter-panel-web/moc.yaml
```

This prints:

- the web preview path for browser verification
- a local `python3 -m http.server` command plus browser URL for HTTP preview
- the Linux desktop app command for the real Tauri window
- the offline headless probe command for terminal-only checks

Run the machine-safe unified runner:

```bash
cargo run -p blocks-cli -- moc run blocks mocs/counter-panel-web/moc.yaml
```

The CLI uses the host's `--headless-probe` path, so this stays safe in terminal-only environments.

Run the host directly:

```bash
cargo run --manifest-path mocs/counter-panel-web/src-tauri/Cargo.toml
```

Probe the host without opening a window:

```bash
cargo --offline run --manifest-path mocs/counter-panel-web/src-tauri/Cargo.toml -- --headless-probe
```

This sample now includes both a minimal local preview at `preview/index.html` and a real Tauri host in `src-tauri/`. The preview and source paths now reuse the same shared frontend block render logic, and `src/index.html` plus `preview/index.html` reuse the same `preview/shell.css` instead of maintaining two separate HTML shells. The current CLI uses the host's headless probe path for automation, while `moc dev` also prints a browser-friendly local HTTP preview command and the direct host command still launches the actual windowed runtime.
