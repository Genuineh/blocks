mod common;

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;

use serde_json::{Value, json};
use tempfile::TempDir;

use blocks_cli::run;
use common::write_block;

fn top_level_keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("json payload should be object")
        .keys()
        .cloned()
        .collect()
}

#[test]
fn runs_demo_echo_block_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
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
    );

    let input_path = temp_dir.path().join("input.json");
    fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

    let output = run(vec![
        "run".to_string(),
        blocks_root.display().to_string(),
        "demo.echo".to_string(),
        input_path.display().to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("\"text\": \"hello\""));
}

#[test]
fn validates_moc_manifest_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
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
    );

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#,
    )
    .expect("manifest should be written");
    let output = run(vec![
        "moc".to_string(),
        "validate".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("valid: echo-pipeline"));
    assert!(output.contains("type=backend_app"));
    assert!(output.contains("backend_mode=console"));
    assert!(output.contains("steps=1"));
}

#[test]
fn validates_descriptor_only_moc_manifest() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: hello-world-console
name: Hello World Console
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    written:
      type: boolean
      required: true
uses:
  blocks:
    - core.console.write_line
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: stdout-line
    channel: stdio
    input_schema:
      text:
        type: string
        required: true
    output_schema:
      written:
        type: boolean
        required: true
verification:
  commands:
    - cargo test
acceptance_criteria:
  - prints the provided text exactly once
"#,
    )
    .expect("manifest should be written");
    let output = run(vec![
        "moc".to_string(),
        "validate".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("valid: hello-world-console"));
    assert!(output.contains("descriptor_only=true"));
}

#[test]
fn runs_moc_validation_flow_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
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
    );

    let manifest_dir = temp_dir.path().join("echo-pipeline");
    fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
    let manifest_path = manifest_dir.join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#,
    )
    .expect("manifest should be written");

    let input_path = temp_dir.path().join("input.json");
    fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

    let output = run(vec![
        "moc".to_string(),
        "verify".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
        input_path.display().to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("\"text\": \"hello\""));
}

#[test]
fn runs_validation_flow_through_moc_run_when_no_launcher_exists() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
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
    );

    let manifest_dir = temp_dir.path().join("echo-pipeline");
    fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
    let manifest_path = manifest_dir.join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#,
    )
    .expect("manifest should be written");
    fs::write(
        manifest_dir.join("input.example.json"),
        r#"{ "text": "hello" }"#,
    )
    .expect("input should be written");

    let output = run(vec![
        "moc".to_string(),
        "run".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect("command should run flow via runtime wrapper");

    assert!(output.contains("\"text\":"));
    assert!(output.contains("trace_id:"));
}

#[test]
fn runs_frontend_moc_from_preview_path() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let moc_root = temp_dir.path().join("counter-panel-web");
    let preview_dir = moc_root.join("preview");
    fs::create_dir_all(&preview_dir).expect("preview dir should be created");
    fs::write(
        preview_dir.join("index.html"),
        "<!doctype html>\n<title>preview</title>\n",
    )
    .expect("preview should be written");

    let manifest_path = moc_root.join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: counter-panel-web
name: Counter Panel Web
type: frontend_app
language: tauri_ts
entry: src/main.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks:
    - ui.counter.mount
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: dom-ready
    channel: webview
    input_schema: {}
    output_schema:
      mounted:
        type: boolean
        required: true
verification:
  commands:
    - review src/main.ts and preview/index.html
acceptance_criteria:
  - mounts a counter into #app
"#,
    )
    .expect("manifest should be written");

    let output = run(vec![
        "moc".to_string(),
        "run".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("frontend preview:"));
    assert!(output.contains("preview/index.html"));
}

#[test]
fn runs_rust_lib_dev_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let moc_root = temp_dir.path().join("hello-message-lib");
    let src_dir = moc_root.join("src");
    fs::create_dir_all(&src_dir).expect("src dir should be created");
    fs::write(
        moc_root.join("Cargo.toml"),
        r#"
[package]
name = "temp-hello-message-lib"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"
"#,
    )
    .expect("cargo manifest should be written");
    fs::write(
        src_dir.join("lib.rs"),
        r#"
pub fn hello_message() -> &'static str {
    "hello world"
}

#[cfg(test)]
mod tests {
    use super::hello_message;

    #[test]
    fn returns_expected_message() {
        assert_eq!(hello_message(), "hello world");
    }
}
"#,
    )
    .expect("lib.rs should be written");
    let manifest_path = moc_root.join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: hello-message-lib
name: Hello Message Lib
type: rust_lib
language: rust
entry: src/lib.rs
public_contract:
  input_schema: {}
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: hello-message
    channel: memory
    input_schema: {}
    output_schema:
      text:
        type: string
        required: true
verification:
  commands:
    - cargo test
acceptance_criteria:
  - returns the fixed hello world message
"#,
    )
    .expect("manifest should be written");

    let output = run(vec![
        "moc".to_string(),
        "dev".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("rust lib dev ok:"));
    assert!(output.contains("Cargo.toml"));
}

#[test]
fn reports_helpful_missing_bind_error_from_moc_verify() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
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
    );

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: missing-bind
name: Missing Bind
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds: []
acceptance_criteria:
  - reports missing bind
"#,
    )
    .expect("manifest should be written");

    let error = run(vec![
        "moc".to_string(),
        "verify".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect_err("command should fail");

    assert!(error.contains("moc verify failed"));
    assert!(error.contains("missing bind for required field echo.text"));
}

#[test]
fn reports_helpful_type_mismatch_error_from_moc_verify() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.number",
        r#"
id: demo.number
name: Demo Number
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  count:
    type: number
    required: true
output_schema:
  count:
    type: number
    required: true
"#,
    );

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: type-mismatch
name: Type Mismatch
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    count:
      type: number
      required: true
uses:
  blocks:
    - demo.number
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: calc
          block: demo.number
      binds:
        - from: input.text
          to: calc.count
acceptance_criteria:
  - reports type mismatch
"#,
    )
    .expect("manifest should be written");

    let error = run(vec![
        "moc".to_string(),
        "verify".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect_err("command should fail");

    assert!(error.contains("moc verify failed"));
    assert!(error.contains("uses the wrong source type"));
    assert!(error.contains("expected number, got string"));
}

#[test]
fn reports_helpful_missing_input_reference_error_from_moc_verify() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
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
    );

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: invalid-reference
name: Invalid Reference
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.missing
          to: echo.text
acceptance_criteria:
  - reports invalid reference
"#,
    )
    .expect("manifest should be written");

    let error = run(vec![
        "moc".to_string(),
        "verify".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect_err("command should fail");

    assert!(error.contains("moc verify failed"));
    assert!(error.contains("uses invalid reference"));
    assert!(error.contains("input.missing"));
}

#[test]
fn reports_frontend_app_human_dev_paths_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let moc_root = temp_dir.path().join("counter-panel-web");
    let preview_dir = moc_root.join("preview");
    fs::create_dir_all(&preview_dir).expect("preview dir should be created");
    fs::write(
        preview_dir.join("index.html"),
        "<!doctype html>\n<title>preview</title>\n",
    )
    .expect("preview should be written");
    let host_dir = moc_root.join("src-tauri");
    fs::create_dir_all(&host_dir).expect("host dir should be created");
    fs::write(
        host_dir.join("Cargo.toml"),
        "[package]\nname = \"counter-panel-web-host\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("cargo manifest should be written");

    let manifest_path = moc_root.join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: counter-panel-web
name: Counter Panel Web
type: frontend_app
language: tauri_ts
entry: src/main.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks:
    - ui.counter.mount
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: dom-ready
    channel: webview
    input_schema: {}
    output_schema:
      mounted:
        type: boolean
        required: true
verification:
  commands:
    - cargo --offline run --manifest-path src-tauri/Cargo.toml -- --headless-probe
acceptance_criteria:
  - renders a counter card into the #app element
"#,
    )
    .expect("manifest should be written");

    let output = run(vec![
        "moc".to_string(),
        "dev".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("frontend app dev: counter-panel-web"));
    assert!(output.contains("web preview:"));
    assert!(output.contains("linux app: cargo run --manifest-path"));
}

#[test]
fn shows_resolved_implementation_path() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
        r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
"#,
    );

    let output = run(vec![
        "show".to_string(),
        blocks_root.display().to_string(),
        "demo.echo".to_string(),
    ])
    .expect("command should succeed");

    assert!(output.contains("implementation:"));
    assert!(output.contains("rust/lib.rs"));
}

#[test]
fn block_diagnose_json_contract_is_stable() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
        r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
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
errors:
  taxonomy:
    - id: invalid_input
    - id: internal_error
"#,
    );

    let input_path = temp_dir.path().join("input.json");
    fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");
    run(vec![
        "run".to_string(),
        blocks_root.display().to_string(),
        "demo.echo".to_string(),
        input_path.display().to_string(),
    ])
    .expect("block run should produce diagnostics");

    let output = run(vec![
        "block".to_string(),
        "diagnose".to_string(),
        blocks_root.display().to_string(),
        "demo.echo".to_string(),
        "--json".to_string(),
    ])
    .expect("diagnose command should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("diagnose output should be valid json");
    assert_eq!(
        top_level_keys(&payload),
        BTreeSet::from([
            "artifact".to_string(),
            "block_id".to_string(),
            "diagnostics_root".to_string(),
            "events".to_string(),
            "execution_id".to_string(),
        ])
    );
    assert_eq!(payload["block_id"], "demo.echo");
    assert!(payload["diagnostics_root"].is_string());
    assert!(payload["execution_id"].is_string());
    assert!(payload["artifact"].is_null());
    let entries = payload["events"]
        .as_array()
        .expect("events should be an array");
    assert!(!entries.is_empty(), "events must not be empty");
    for entry in entries {
        let keys = entry
            .as_object()
            .expect("event entry should be object")
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert!(keys.contains("block_id"));
        assert!(keys.contains("event"));
        assert!(keys.contains("execution_id"));
        assert!(keys.contains("timestamp_ms"));
        assert!(entry["block_id"].is_string());
        assert!(entry["event"].is_string());
        assert!(entry["execution_id"].is_string());
        assert!(entry["timestamp_ms"].is_u64());
    }
}

#[test]
fn moc_diagnose_json_trace_chain_contract_is_stable() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
        r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
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
    );

    let manifest_dir = temp_dir.path().join("echo-pipeline");
    fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
    let manifest_path = manifest_dir.join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
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
acceptance_criteria:
  - echoes the provided text twice
"#,
    )
    .expect("manifest should be written");
    let input_path = manifest_dir.join("input.example.json");
    fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

    let verify_output = run(vec![
        "moc".to_string(),
        "verify".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
        input_path.display().to_string(),
    ])
    .expect("moc verify should succeed");
    let trace_id = verify_output
        .lines()
        .find_map(|line| line.strip_prefix("trace_id: "))
        .expect("trace_id should be present in verify output")
        .to_string();

    let output = run(vec![
        "moc".to_string(),
        "diagnose".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
        "--trace-id".to_string(),
        trace_id.clone(),
        "--json".to_string(),
    ])
    .expect("moc diagnose command should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("moc diagnose output should be valid json");
    assert_eq!(
        top_level_keys(&payload),
        BTreeSet::from([
            "artifacts".to_string(),
            "diagnostics_root".to_string(),
            "events".to_string(),
            "manifest_path".to_string(),
            "moc_id".to_string(),
            "trace_id".to_string(),
        ])
    );
    assert_eq!(payload["trace_id"].as_str(), Some(trace_id.as_str()));
    assert_eq!(payload["moc_id"].as_str(), Some("echo-pipeline"));
    assert!(payload["diagnostics_root"].is_string());
    assert!(payload["manifest_path"].is_string());
    assert!(payload["artifacts"].is_array());

    let entries = payload["events"]
        .as_array()
        .expect("events should be an array");
    assert!(entries.len() >= 2);
    for entry in entries {
        assert_eq!(entry["trace_id"].as_str(), Some(trace_id.as_str()));
        assert_eq!(entry["moc_id"].as_str(), Some("echo-pipeline"));
        assert!(entry["execution_id"].as_str().is_some());
        assert!(entry["event"].is_string());
        assert!(entry["timestamp_ms"].is_u64());
    }
}

#[test]
fn moc_diagnose_prefers_direct_moc_id_match_before_block_fallback() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
        r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
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
    );

    let primary_dir = temp_dir.path().join("primary");
    fs::create_dir_all(&primary_dir).expect("primary dir should be created");
    let primary_manifest = primary_dir.join("moc.yaml");
    fs::write(
        &primary_manifest,
        r#"
id: primary-moc
name: Primary Moc
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes text
"#,
    )
    .expect("primary manifest should be written");
    let primary_input = primary_dir.join("input.example.json");
    fs::write(&primary_input, r#"{ "text": "from-primary" }"#).expect("input should be written");

    let verify_output = run(vec![
        "moc".to_string(),
        "verify".to_string(),
        blocks_root.display().to_string(),
        primary_manifest.display().to_string(),
        primary_input.display().to_string(),
    ])
    .expect("primary moc verify should succeed");
    let primary_trace_id = verify_output
        .lines()
        .find_map(|line| line.strip_prefix("trace_id: "))
        .expect("trace_id should be present in verify output")
        .to_string();

    let diagnostics_root = temp_dir.path().join(".blocks").join("diagnostics");
    fs::create_dir_all(&diagnostics_root).expect("diagnostics dir should be created");
    let events_path = diagnostics_root.join("events.jsonl");
    let mut events_file = fs::OpenOptions::new()
        .append(true)
        .open(&events_path)
        .expect("events file should exist after moc verify");
    let fallback_event = json!({
        "timestamp_ms": u64::MAX,
        "level": "INFO",
        "event": "block.execution.success",
        "block_id": "demo.echo",
        "block_version": "1.0.0",
        "execution_id": "exec-fallback-test",
        "trace_id": "trace-fallback-test",
        "duration_ms": 1
    });
    writeln!(events_file, "{fallback_event}").expect("fallback event should be appended");

    let diagnose_output = run(vec![
        "moc".to_string(),
        "diagnose".to_string(),
        blocks_root.display().to_string(),
        primary_manifest.display().to_string(),
        "--json".to_string(),
    ])
    .expect("moc diagnose should succeed");
    let payload: Value =
        serde_json::from_str(&diagnose_output).expect("moc diagnose output should be valid json");
    assert_eq!(
        payload["trace_id"].as_str(),
        Some(primary_trace_id.as_str())
    );
    assert_eq!(payload["moc_id"].as_str(), Some("primary-moc"));
}

#[test]
fn redacts_sensitive_values_in_moc_diagnose_artifacts() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    write_block(
        &blocks_root,
        "demo.echo",
        r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
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
  token:
    type: string
    required: true
"#,
    );
    let manifest_dir = temp_dir.path().join("echo-pipeline");
    fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
    let manifest_path = manifest_dir.join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: first
          block: demo.echo
      binds:
        - from: input.text
          to: first.text
acceptance_criteria:
  - echoes the provided text
"#,
    )
    .expect("manifest should be written");
    let input_path = manifest_dir.join("input.example.json");
    fs::write(&input_path, r#"{ "text": "Bearer super-secret-token" }"#)
        .expect("input should be written");

    let _verify_error = run(vec![
        "moc".to_string(),
        "verify".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
        input_path.display().to_string(),
    ])
    .expect_err("moc verify should fail and emit diagnostics");

    let output = run(vec![
        "moc".to_string(),
        "diagnose".to_string(),
        blocks_root.display().to_string(),
        manifest_path.display().to_string(),
        "--json".to_string(),
    ])
    .expect("moc diagnose command should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("moc diagnose output should be valid json");
    let artifacts = payload["artifacts"]
        .as_array()
        .expect("artifacts should be an array");
    let artifact = artifacts
        .iter()
        .find_map(|item| item.as_object())
        .expect("at least one artifact should be present");
    assert!(artifact.get("execution_id").is_some_and(Value::is_string));
    assert!(artifact.get("block_id").is_some_and(Value::is_string));
    assert!(artifact.get("error").is_some_and(Value::is_object));
    assert!(artifact["error"]["error_id"].is_string());
    assert!(artifact["error"]["message"].is_string());
    assert!(artifact.get("environment").is_some_and(Value::is_object));
    assert!(artifact["environment"]["runtime_mode"].is_string());
    assert!(artifact.get("input_snapshot").is_some_and(Value::is_object));
    if let Some(timestamp_ms) = artifact.get("timestamp_ms") {
        assert!(timestamp_ms.is_u64());
    }
    if let Some(output_snapshot) = artifact.get("output_snapshot") {
        assert!(output_snapshot.is_object());
    }
    assert_eq!(
        artifact["input_snapshot"]["text"].as_str(),
        Some("***REDACTED***")
    );
}
