# hello-pipeline

Minimal file-based app that writes text, then reads it back through blocks.

Validate the descriptor:

```bash
cargo run -p blocks-cli -- compose validate blocks apps/hello-pipeline/app.yaml
```

Run the real backend launcher:

```bash
cargo run --manifest-path apps/hello-pipeline/backend/Cargo.toml -- blocks apps/hello-pipeline/app.yaml apps/hello-pipeline/input.example.json
```
