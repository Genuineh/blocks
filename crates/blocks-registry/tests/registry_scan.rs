use std::fs;

use blocks_registry::{Registry, RegistryError};
use tempfile::TempDir;

fn write_contract(root: &TempDir, dir_name: &str, id: &str, name: &str) {
    let block_dir = root.path().join(dir_name);
    let rust_dir = block_dir.join("rust");
    fs::create_dir_all(&rust_dir).expect("block dir should be created");
    fs::write(
        block_dir.join("block.yaml"),
        format!(
            "id: {id}\nname: {name}\nimplementation:\n  kind: rust\n  entry: rust/lib.rs\n  target: shared\ninput_schema:\n  value:\n    type: string\n    required: true\n"
        ),
    )
    .expect("contract should be written");
    fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
}

#[test]
fn errors_when_blocks_root_is_missing() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let missing_root = temp_dir.path().join("missing");

    let result = Registry::load_from_root(&missing_root);

    assert!(matches!(result, Err(RegistryError::MissingRoot(path)) if path == missing_root));
}

#[test]
fn errors_on_duplicate_block_ids() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    write_contract(&temp_dir, "first", "demo.echo", "Echo A");
    write_contract(&temp_dir, "second", "demo.echo", "Echo B");

    let result = Registry::load_from_root(temp_dir.path());

    assert!(matches!(result, Err(RegistryError::DuplicateBlockId(id)) if id == "demo.echo"));
}

#[test]
fn discovers_blocks_and_supports_search() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    write_contract(&temp_dir, "http", "core.http.get", "HTTP Get");
    write_contract(&temp_dir, "file", "core.fs.read_text", "Read Text");

    let registry = Registry::load_from_root(temp_dir.path()).expect("registry should load");

    assert_eq!(registry.list().len(), 2);
    let block = registry
        .get("core.http.get")
        .expect("http block should exist");
    assert!(block.implementation_path.ends_with("rust/lib.rs"));
    assert_eq!(registry.search("read").len(), 1);
    assert_eq!(registry.search("http").len(), 1);
}
