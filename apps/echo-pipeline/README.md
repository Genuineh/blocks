# echo-pipeline

Minimal serial app used to verify descriptor validation and backend launch.

Validate the descriptor:

```bash
cargo run -p blocks-cli -- compose validate blocks apps/echo-pipeline/app.yaml
```

Run the real backend launcher:

```bash
cargo run --manifest-path apps/echo-pipeline/backend/Cargo.toml -- blocks apps/echo-pipeline/app.yaml apps/echo-pipeline/input.example.json
```
