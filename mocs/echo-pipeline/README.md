# echo-pipeline

Minimal serial moc used to verify optional flow validation and backend launch.

The backend entrypoint now directly depends on the Rust crate for `demo.echo` and calls it twice from `main`.

Validate the descriptor:

```bash
cargo run -p blocks-cli -- moc validate blocks mocs/echo-pipeline/moc.yaml
```

Run the BCL assist path for the same moc:

```bash
mkdir -p .tmp
cargo run -p blocks-cli -- moc bcl validate blocks mocs/echo-pipeline/moc.bcl --json
cargo run -p blocks-cli -- moc bcl plan blocks mocs/echo-pipeline/moc.bcl --json
cargo run -p blocks-cli -- moc bcl emit blocks mocs/echo-pipeline/moc.bcl --out .tmp/echo-pipeline.generated.yaml --check-against mocs/echo-pipeline/moc.yaml
```

This keeps `moc.yaml` as the runtime authority while proving that `moc.bcl` can emit the same canonical descriptor.

Run through the unified moc runner:

```bash
cargo run -p blocks-cli -- moc run blocks mocs/echo-pipeline/moc.yaml
```

Run the real backend launcher with the default sample input:

```bash
cargo run --manifest-path mocs/echo-pipeline/backend/Cargo.toml
```

Or run it with a custom input file:

```bash
cargo run --manifest-path mocs/echo-pipeline/backend/Cargo.toml -- /path/to/input.json
```
