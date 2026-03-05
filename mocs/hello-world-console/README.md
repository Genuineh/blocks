# hello-world-console

Minimal moc that demonstrates a free `moc.main`.

The backend entrypoint is intentionally minimal: it directly depends on the Rust block crate for `core.console.write_line` and executes that block from `main`.

The backend also links the sibling `rust_lib` moc [hello-message-lib](../hello-message-lib/README.md) and uses it to produce the final `hello world` text before handing that text to the public console block.

It also includes a sample private block layout under `internal_blocks/hello_world.message/` to fix the expected directory structure for moc-local blocks.

`moc.yaml` is still present for descriptor validation, but the runnable sample itself needs no input file and no manifest argument. The visible message comes from the linked library moc, not from external input.

`block.yaml` for `core.console.write_line` remains the AI-facing descriptor and validation source. The actual program path is ordinary Rust code calling the block crate directly.

Validate the descriptor:

```bash
cargo run -p blocks-cli -- moc validate blocks mocs/hello-world-console/moc.yaml
```

Run it through the unified moc runner:

```bash
cargo run -p blocks-cli -- moc run blocks mocs/hello-world-console/moc.yaml
```

Run the backend:

```bash
cargo run --manifest-path mocs/hello-world-console/backend/Cargo.toml
```
