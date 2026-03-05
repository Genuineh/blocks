# hello-panel-lib

Minimal `frontend_lib` moc example.

It exports a reusable frontend function from `src/index.ts`, while `moc.yaml` remains the descriptor used for AI guidance and validation.

Structure:

- `src/index.ts`: reusable frontend library entry
- `preview/index.html`: toolchain-free preview for the library behavior

Validate the descriptor:

```bash
cargo run -p blocks-cli -- moc validate blocks mocs/hello-panel-lib/moc.yaml
```

Run the local development preview:

```bash
cargo run -p blocks-cli -- moc dev blocks mocs/hello-panel-lib/moc.yaml
```

The current MVP keeps this as a source-first frontend library with a static preview path, not a packaged frontend build. The preview reuses the same shared frontend block render code as the source path, and `moc dev` now prints a browser-friendly local HTTP preview command alongside the resolved preview file.
