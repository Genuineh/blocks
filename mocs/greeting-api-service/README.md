# greeting-api-service

Minimal `backend_app(service)` moc that exposes a real HTTP contract for the proof slice.

It serves `GET /api/v1/greeting` and returns:

```json
{
  "title": "Hello from blocks",
  "message": "The moc model can carry a real backend-to-frontend slice."
}
```

Run the backend on the default manual demo port (`4318`):

```bash
cargo run --manifest-path mocs/greeting-api-service/backend/Cargo.toml
```

Override the port when needed:

```bash
GREETING_API_PORT=4500 cargo run --manifest-path mocs/greeting-api-service/backend/Cargo.toml
```

Automated verification:

- `cargo test --manifest-path mocs/greeting-api-service/backend/Cargo.toml`

The automated tests validate the HTTP contract on an ephemeral port. They do not keep a long-running server process alive after the assertions complete.
