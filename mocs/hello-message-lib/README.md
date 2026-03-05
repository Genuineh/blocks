# hello-message-lib

Minimal `rust_lib` moc.

It exposes a `hello-message` protocol with a memory channel and provides a tiny Rust library entrypoint at `src/lib.rs`.

The sibling `hello-world-console` moc depends on this descriptor and validates protocol compatibility against it.

Validate the descriptor:

```bash
cargo run -p blocks-cli -- moc validate blocks mocs/hello-message-lib/moc.yaml
```

Run the unified local development command:

```bash
cargo run -p blocks-cli -- moc dev blocks mocs/hello-message-lib/moc.yaml
```

Or run the library tests directly:

```bash
cargo test -p hello-message-lib
```
