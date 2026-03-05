# hello-pipeline

Minimal file-based moc that writes text, then reads it back through blocks.

The backend entrypoint now directly depends on the Rust block crates for `core.fs.write_text` and `core.fs.read_text`.

Validate the descriptor:

```bash
cargo run -p blocks-cli -- moc validate blocks mocs/hello-pipeline/moc.yaml
```

Run through the unified moc runner:

```bash
cargo run -p blocks-cli -- moc run blocks mocs/hello-pipeline/moc.yaml
```

Run the real backend launcher with the default sample input:

```bash
cargo run --manifest-path mocs/hello-pipeline/backend/Cargo.toml
```

Or run it with a custom input file:

```bash
cargo run --manifest-path mocs/hello-pipeline/backend/Cargo.toml -- /path/to/input.json
```
