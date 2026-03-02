use std::fs;

use blocks_composer::{AppManifest, ComposeError, Composer};
use blocks_runtime::{BlockExecutionError, BlockRunner};
use serde_json::{Value, json};
use tempfile::TempDir;

struct EchoRunner;

impl BlockRunner for EchoRunner {
    fn run(&self, block_id: &str, input: &Value) -> Result<Value, BlockExecutionError> {
        match block_id {
            "demo.echo" => Ok(json!({
                "text": input.get("text").cloned().unwrap_or(Value::Null)
            })),
            other => Err(BlockExecutionError::new(format!(
                "unexpected block execution: {other}"
            ))),
        }
    }
}

fn write_demo_echo_block(root: &TempDir) {
    let block_dir = root.path().join("demo.echo");
    fs::create_dir_all(&block_dir).expect("block dir should be created");
    fs::write(
        block_dir.join("block.yaml"),
        r#"
id: demo.echo
name: Demo Echo
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

    let result =
        Composer::new().execute(&manifest, &json!({"text": "hello"}), &registry, &EchoRunner);

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

    let result = Composer::new().execute(&manifest, &json!({"text": 42}), &registry, &EchoRunner);

    assert!(matches!(
        result,
        Err(ComposeError::TypeMismatch { from, to, .. })
        if from == "input.text" && to == "echo.text"
    ));
}

#[test]
fn executes_serial_flow_and_returns_last_step_output() {
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

    let result = Composer::new()
        .execute(&manifest, &json!({"text": "hello"}), &registry, &EchoRunner)
        .expect("composer should succeed");

    assert_eq!(result.last_step_id, "second");
    assert_eq!(result.output, json!({"text": "hello"}));
}
