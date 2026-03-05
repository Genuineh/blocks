# hello-panel-web

Minimal `frontend_app` moc example for the Tauri + TypeScript boundary.

It directly imports the frontend block `ui.dom.mount_text` from code, while `moc.yaml` remains the descriptor used for AI guidance and validation.

Structure:

- `src/main.ts`: frontend entrypoint
- `preview/index.html`: toolchain-free static preview
- `src-tauri/README.md`: Tauri host boundary note

Validate the descriptor:

```bash
cargo run -p blocks-cli -- moc validate blocks mocs/hello-panel-web/moc.yaml
```

Get the human-facing browser preview path and local HTTP preview command:

```bash
cargo run -p blocks-cli -- moc dev blocks mocs/hello-panel-web/moc.yaml
```

Run the machine-safe preview path:

```bash
cargo run -p blocks-cli -- moc run blocks mocs/hello-panel-web/moc.yaml
```

This sample now includes a minimal local preview at `preview/index.html`, and the preview reuses the same shared frontend block render code as the source path. `moc dev` now prints both the preview file path and a browser-friendly local HTTP preview command, while `moc run` remains the terminal-safe check.
