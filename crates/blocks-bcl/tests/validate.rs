use std::fs;

use blocks_bcl::{success_report, validate_file};
use tempfile::TempDir;

fn write_block(root: &std::path::Path, dir_name: &str, body: &str) {
    let block_dir = root.join(dir_name);
    let rust_dir = block_dir.join("rust");
    fs::create_dir_all(&rust_dir).expect("block dir should be created");
    fs::write(block_dir.join("block.yaml"), body).expect("contract should be written");
    fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
}

fn valid_block_contract(
    id: &str,
    input_type: &str,
    output_type: &str,
    input_required: bool,
) -> String {
    format!(
        r#"
id: {id}
name: Test Block
version: 0.1.0
status: candidate
owner: blocks-core-team
purpose: test block
scope:
  - test scope
non_goals:
  - test non-goal
inputs:
  - name: value
    description: input
preconditions:
  - input exists
outputs:
  - name: value
    description: output
postconditions:
  - output exists
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
dependencies:
  runtime:
    - std
side_effects:
  - none
timeouts:
  default_ms: 100
resource_limits:
  memory_mb: 16
failure_modes:
  - id: invalid_input
    when: invalid input
error_codes:
  - invalid_input
recovery_strategy:
  - retry
verification:
  automated:
    - cargo test
evaluation:
  quality_gates:
    - stable
acceptance_criteria:
  - works
debug:
  enabled_in_dev: true
  emits_structured_logs: true
  log_fields:
    - execution_id
observe:
  metrics:
    - execution_total
  emits_failure_artifact: true
  artifact_policy:
    mode: on_failure
errors:
  taxonomy:
    - id: invalid_input
    - id: internal_error
input_schema:
  value:
    type: {input_type}
    required: {input_required}
output_schema:
  value:
    type: {output_type}
    required: true
"#
    )
}

#[test]
fn validates_descriptor_only_bcl() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let moc_dir = temp_dir.path().join("hello");
    fs::create_dir_all(&moc_dir).expect("moc dir should be created");
    let source_path = moc_dir.join("moc.bcl");
    fs::write(
        &source_path,
        r#"
moc hello {
  name "Hello";
  type backend_app(console);
  language rust;
  entry "backend/src/main.rs";
  uses { }
  depends_on_mocs { }
  protocols { }
  verification { command "cargo test"; }
  accept "works";
}
"#,
    )
    .expect("bcl should be written");

    let validated = validate_file(
        &blocks_root.display().to_string(),
        &source_path.display().to_string(),
    )
    .expect("bcl should validate");
    assert_eq!(validated.moc_id, "hello");
    assert_eq!(
        success_report(&source_path.display().to_string()).status,
        "ok"
    );
}

#[test]
fn reports_syntax_error_with_span() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let source_path = temp_dir.path().join("broken.bcl");
    fs::write(
        &source_path,
        r#"
moc bad {
  name "Bad"
}
"#,
    )
    .expect("bcl should be written");

    let report = validate_file(
        &blocks_root.display().to_string(),
        &source_path.display().to_string(),
    )
    .expect_err("syntax error expected");
    assert_eq!(report.status, "error");
    assert_eq!(report.rule_results[0].rule_id, "BCL-SYNTAX-001");
    assert!(report.rule_results[0].span.line >= 1);
}

#[test]
fn reports_bind_invalid_reference() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
        &valid_block_contract("demo.echo", "string", "string", false),
    );

    let source_path = temp_dir.path().join("invalid-ref.bcl");
    fs::write(
        &source_path,
        r#"
moc invalid_ref {
  name "Invalid Ref";
  type backend_app(console);
  language rust;
  entry "backend/src/main.rs";
  input {
    value: string required;
  }
  output {
    value: string required;
  }
  uses { block demo.echo; }
  depends_on_mocs { }
  protocols { }
  verification {
    entry flow plan {
      step first = demo.echo;
      step second = demo.echo;
      bind input.value -> first.value;
      bind missing.value -> second.value;
    }
  }
  accept "fails";
}
"#,
    )
    .expect("bcl should be written");

    let report = validate_file(
        &blocks_root.display().to_string(),
        &source_path.display().to_string(),
    )
    .expect_err("invalid reference expected");
    assert_eq!(report.rule_results[0].rule_id, "BCL-SEMA-002");
}

#[test]
fn reports_bind_type_mismatch() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
        &valid_block_contract("demo.echo", "string", "string", false),
    );
    write_block(
        &blocks_root,
        "demo.number",
        &valid_block_contract("demo.number", "number", "number", true),
    );

    let source_path = temp_dir.path().join("type-mismatch.bcl");
    fs::write(
        &source_path,
        r#"
moc type_mismatch {
  name "Type Mismatch";
  type backend_app(console);
  language rust;
  entry "backend/src/main.rs";
  input {
    value: string required;
  }
  uses { block demo.echo; block demo.number; }
  depends_on_mocs { }
  protocols { }
  verification {
    entry flow plan {
      step first = demo.echo;
      step second = demo.number;
      bind input.value -> first.value;
      bind first.value -> second.value;
    }
  }
  accept "fails";
}
"#,
    )
    .expect("bcl should be written");

    let report = validate_file(
        &blocks_root.display().to_string(),
        &source_path.display().to_string(),
    )
    .expect_err("type mismatch expected");
    assert_eq!(report.rule_results[0].rule_id, "BCL-SEMA-004");
}

#[test]
fn reports_protocol_mismatch() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let mocs_root = temp_dir.path().join("mocs");
    let remote_dir = mocs_root.join("remote-service");
    fs::create_dir_all(&remote_dir).expect("remote dir should be created");
    fs::write(
        remote_dir.join("moc.yaml"),
        r#"
id: remote-service
name: Remote Service
type: backend_app
backend_mode: service
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: greeting-http
    channel: http
    input_schema: {}
    output_schema:
      message:
        type: string
        required: true
verification:
  commands:
    - cargo test
acceptance_criteria:
  - works
"#,
    )
    .expect("remote moc should be written");

    let local_dir = mocs_root.join("local-panel");
    fs::create_dir_all(&local_dir).expect("local dir should be created");
    let source_path = local_dir.join("moc.bcl");
    fs::write(
        &source_path,
        r#"
moc local_panel {
  name "Local Panel";
  type frontend_app;
  language tauri_ts;
  entry "src/main.ts";
  uses { }
  depends_on_mocs { moc "remote-service" via greeting-http; }
  protocols {
    protocol greeting-http {
      channel http;
      input { }
      output { title: string required; }
    }
  }
  verification { command "npm test"; }
  accept "fails";
}
"#,
    )
    .expect("bcl should be written");

    let report = validate_file(
        &blocks_root.display().to_string(),
        &source_path.display().to_string(),
    )
    .expect_err("protocol mismatch expected");
    assert_eq!(report.rule_results[0].rule_id, "BCL-PROTO-001");
}
