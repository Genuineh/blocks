# greeting-panel-web

Minimal `frontend_app` moc that fetches `greeting-api-service` and renders the returned greeting.

The frontend uses the shared `greeting-http` protocol declaration and keeps its request to a simple `GET`, so the backend can safely answer with `Access-Control-Allow-Origin: *` and no preflight is required.

Automated checks:

- `node --test mocs/greeting-panel-web/tests/greeting_panel.test.mjs`
- `cargo --offline run --manifest-path mocs/greeting-panel-web/src-tauri/Cargo.toml -- --headless-probe`

Those checks prove:

- the frontend state logic covers loading, success, and error transitions
- the Tauri host is wired to the preview assets and can start its bounded headless probe path

They do **not** prove the real backend-to-frontend fetch+render join in one browser window.

Manual verification for the real join:

1. Start the backend: `cargo run --manifest-path mocs/greeting-api-service/backend/Cargo.toml`
2. In a second terminal, serve the repository root: `python3 -m http.server --directory . 4173`
3. Open `http://127.0.0.1:4173/mocs/greeting-panel-web/preview/`

The page should move from a loading state to the greeting returned by `GET /api/v1/greeting`.

For the desktop host path, run:

```bash
cargo run --manifest-path mocs/greeting-panel-web/src-tauri/Cargo.toml
```
