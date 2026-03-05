# core.console.write_line

Writes the provided `text` to standard output and returns `written: true`.

This block now also ships as a standalone Rust crate at `blocks/core.console.write_line/rust/`.

That means a Rust `moc` can depend on it directly as code, while `block.yaml` remains the descriptor used by AI, registry scanning, and contract validation.
