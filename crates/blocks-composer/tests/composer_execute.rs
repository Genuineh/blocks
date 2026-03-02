use std::collections::BTreeMap;
use std::fs;

use blocks_composer::{AppManifest, ComposeError, Composer};
use serde_json::json;
use tempfile::TempDir;

fn write_demo_echo_block(root: &TempDir) {
    let block_dir = root.path().join("demo.echo");
    let rust_dir = block_dir.join("rust");
    fs::create_dir_all(&rust_dir).expect("block dir should be created");
    fs::write(
        block_dir.join("block.yaml"),
        r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
    )
    .expect("contract should be written");
    fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
}

#[test]
fn rejects_manifest_when_required_step_input_has_no_bind() {
    let blocks_root = TempDir::new().expect("temp dir should be created");
    write_demo_echo_block(&blocks_root);
    let registry = blocks_registry::Registry::load_from_root(blocks_root.path())
        .expect("registry should load");
    let manifest = AppManifest::from_yaml_str(
        r#"
name: missing-bind
entry: main
input_schema:
  text:
    type: string
    required: true
flows:
  - id: main
    steps:
      - id: echo
        block: demo.echo
    binds: []
"#,
    )
    .expect("manifest should parse");

    let result = Composer::new().plan(&manifest, &registry);

    assert!(matches!(
        result,
        Err(ComposeError::MissingBind { step_id, field })
        if step_id == "echo" && field == "text"
    ));
}

#[test]
fn rejects_manifest_when_bind_types_are_incompatible() {
    let blocks_root = TempDir::new().expect("temp dir should be created");
    write_demo_echo_block(&blocks_root);
    let registry = blocks_registry::Registry::load_from_root(blocks_root.path())
        .expect("registry should load");
    let manifest = AppManifest::from_yaml_str(
        r#"
name: type-mismatch
entry: main
input_schema:
  text:
    type: number
    required: true
flows:
  - id: main
    steps:
      - id: echo
        block: demo.echo
    binds:
      - from: input.text
        to: echo.text
"#,
    )
    .expect("manifest should parse");

    let result = Composer::new().plan(&manifest, &registry);

    assert!(matches!(
        result,
        Err(ComposeError::TypeMismatch { from, to, .. })
        if from == "input.text" && to == "echo.text"
    ));
}

#[test]
fn builds_a_serial_execution_plan() {
    let blocks_root = TempDir::new().expect("temp dir should be created");
    write_demo_echo_block(&blocks_root);
    let registry = blocks_registry::Registry::load_from_root(blocks_root.path())
        .expect("registry should load");
    let manifest = AppManifest::from_yaml_str(
        r#"
name: echo-pipeline
entry: main
input_schema:
  text:
    type: string
    required: true
flows:
  - id: main
    steps:
      - id: first
        block: demo.echo
      - id: second
        block: demo.echo
    binds:
      - from: input.text
        to: first.text
      - from: first.text
        to: second.text
"#,
    )
    .expect("manifest should parse");

    let plan = Composer::new()
        .plan(&manifest, &registry)
        .expect("planner should succeed");

    assert_eq!(plan.last_step_id, "second");
    assert_eq!(plan.steps.len(), 2);

    let first_input = plan.steps[0]
        .build_input(&json!({ "text": "hello" }), &BTreeMap::new())
        .expect("first step input should build");
    assert_eq!(first_input.get("text"), Some(&json!("hello")));

    let mut step_outputs = BTreeMap::new();
    step_outputs.insert("first".to_string(), json!({ "text": "hello" }));

    let second_input = plan.steps[1]
        .build_input(&json!({ "text": "hello" }), &step_outputs)
        .expect("second step input should build");
    assert_eq!(second_input.get("text"), Some(&json!("hello")));
}
