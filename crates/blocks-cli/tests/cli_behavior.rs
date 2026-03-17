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

fn write_evidence_block(root: &std::path::Path, block_id: &str) -> std::path::PathBuf {
    write_block(
        root,
        block_id,
        r#"
id: demo.evidence
name: Demo Evidence
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
verification:
  automated:
    - echo fallback-verification
evaluation:
  commands:
    - echo fallback-evaluation
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

    let block_root = root.join(block_id);
    fs::create_dir_all(block_root.join("tests")).expect("tests dir should be created");
    fs::create_dir_all(block_root.join("examples")).expect("examples dir should be created");
    fs::create_dir_all(block_root.join("evaluators")).expect("evaluators dir should be created");
    fs::create_dir_all(block_root.join("fixtures")).expect("fixtures dir should be created");
    fs::write(
        block_root.join("tests").join("run.sh"),
        "#!/usr/bin/env sh\nset -eu\ntest -f rust/lib.rs\n",
    )
    .expect("tests runner should be written");
    fs::write(
        block_root.join("examples").join("run.sh"),
        "#!/usr/bin/env sh\nset -eu\ngrep -q 'hello' examples/success.input.json\n",
    )
    .expect("examples runner should be written");
    fs::write(
        block_root.join("evaluators").join("run.sh"),
        "#!/usr/bin/env sh\nset -eu\ngrep -q 'hello' fixtures/success.input.json\n",
    )
    .expect("evaluator runner should be written");
    fs::write(
        block_root.join("examples").join("success.input.json"),
        r#"{ "text": "hello from example" }"#,
    )
    .expect("example input should be written");
    fs::write(
        block_root.join("fixtures").join("success.input.json"),
        r#"{ "text": "hello from fixture" }"#,
    )
    .expect("fixture input should be written");

    block_root
}

fn write_runtime_block(root: &std::path::Path, block_id: &str, target: &str) -> std::path::PathBuf {
    write_block(
        root,
        block_id,
        &format!(
            r#"
id: {block_id}
name: Runtime Block
implementation:
  kind: rust
  entry: rust/lib.rs
  target: {target}
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#
        ),
    );
    let block_root = root.join(block_id);
    fs::create_dir_all(block_root.join("fixtures")).expect("fixtures dir should be created");
    fs::write(
        block_root.join("fixtures").join("success.input.json"),
        r#"{ "text": "hello from runtime fixture" }"#,
    )
    .expect("runtime fixture should be written");
    block_root
}

fn file_registry_release_root(
    registry_root: &std::path::Path,
    package_id: &str,
    version: &str,
) -> std::path::PathBuf {
    registry_root
        .join(package_id.replace('.', "__"))
        .join(version)
}

fn write_file_registry_package(
    registry_root: &std::path::Path,
    package_id: &str,
    version: &str,
) -> std::path::PathBuf {
    let release_root = file_registry_release_root(registry_root, package_id, version);
    fs::create_dir_all(&release_root).expect("release root should be created");
    fs::write(
        release_root.join("package.yaml"),
        format!(
            r#"
api_version: blocks.pkg/v1
kind: block
id: {package_id}
version: {version}
descriptor:
  path: block.yaml
dependencies: []
"#
        ),
    )
    .expect("package manifest should be written");
    fs::write(
        release_root.join("block.yaml"),
        format!("id: {package_id}\n"),
    )
    .expect("descriptor should be written");
    release_root
}

fn write_package_consumer(
    workspace_root: &std::path::Path,
    package_id: &str,
    dependency_id: &str,
    req: &str,
) -> std::path::PathBuf {
    let package_root = workspace_root.join(package_id.replace('.', "-"));
    fs::create_dir_all(&package_root).expect("package root should be created");
    fs::write(
        package_root.join("package.yaml"),
        format!(
            r#"
api_version: blocks.pkg/v1
kind: block
id: {package_id}
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: {dependency_id}
    kind: block
    req: {req}
"#
        ),
    )
    .expect("package manifest should be written");
    fs::write(
        package_root.join("block.yaml"),
        format!("id: {package_id}\n"),
    )
    .expect("descriptor should be written");
    package_root
}

fn write_workspace_block_package(
    packages_root: &std::path::Path,
    package_id: &str,
    version: &str,
) -> std::path::PathBuf {
    let package_root = packages_root.join(package_id.replace('.', "-"));
    fs::create_dir_all(package_root.join("rust")).expect("package rust dir should be created");
    fs::write(
        package_root.join("package.yaml"),
        format!(
            r#"
api_version: blocks.pkg/v1
kind: block
id: {package_id}
version: {version}
descriptor:
  path: block.yaml
dependencies: []
"#
        ),
    )
    .expect("package manifest should be written");
    fs::write(
        package_root.join("block.yaml"),
        format!(
            r#"
id: {package_id}
name: Packaged Echo
version: {version}
status: candidate
owner: blocks-core-team
purpose: package-aware test block
scope:
  - test scope
non_goals:
  - test non-goal
inputs:
  - name: text
    description: input
preconditions:
  - input exists
outputs:
  - name: text
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
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#
        ),
    )
    .expect("block contract should be written");
    fs::write(
        package_root.join("rust").join("lib.rs"),
        "// packaged block fixture\n",
    )
    .expect("block implementation should be written");
    package_root
}

fn write_bcl_package_consumer(
    packages_root: &std::path::Path,
    package_id: &str,
    dependency_id: &str,
    req: &str,
) -> std::path::PathBuf {
    let package_root = packages_root.join(package_id.replace('.', "-"));
    fs::create_dir_all(&package_root).expect("bcl package root should be created");
    fs::write(
        package_root.join("package.yaml"),
        format!(
            r#"
api_version: blocks.pkg/v1
kind: bcl
id: {package_id}
version: 0.1.0
descriptor:
  path: moc.bcl
dependencies:
  - id: {dependency_id}
    kind: block
    req: {req}
"#
        ),
    )
    .expect("bcl package manifest should be written");
    fs::write(
        package_root.join("moc.bcl"),
        format!(
            r#"
moc packaged_flow {{
  name "Packaged Flow";
  type backend_app(console);
  language rust;
  entry "backend/src/main.rs";
  input {{
    text: string required;
  }}
  output {{
    text: string required;
  }}
  uses {{ block {dependency_id}; }}
  depends_on_mocs {{ }}
  protocols {{ }}
  verification {{
    command "cargo test";
    entry flow plan {{
      step echo = {dependency_id};
      bind input.text -> echo.text;
    }}
  }}
  accept "works";
}}
"#
        ),
    )
    .expect("bcl source should be written");
    package_root
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

[workspace]
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

#[test]
fn validates_bcl_descriptor_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let source_path = temp_dir.path().join("moc.bcl");
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

    let output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "validate".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--json".to_string(),
    ])
    .expect("bcl validate should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl validate output should be valid json");
    let keys = top_level_keys(&payload);
    assert_eq!(
        keys,
        BTreeSet::from([
            "rule_results".to_string(),
            "source".to_string(),
            "status".to_string(),
        ])
    );
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["rule_results"], json!([]));
}

#[test]
fn validates_bcl_descriptor_from_top_level_namespace_with_inferred_blocks_root() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let workspace_root = temp_dir.path().join("workspace");
    let blocks_root = workspace_root.join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let moc_root = workspace_root.join("mocs").join("hello");
    fs::create_dir_all(&moc_root).expect("moc root should be created");

    fs::write(
        moc_root.join("moc.bcl"),
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

    let output = run(vec![
        "bcl".to_string(),
        "check".to_string(),
        moc_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("top-level bcl check should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl check output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(
        payload["source"],
        moc_root.join("moc.bcl").display().to_string()
    );
}

#[test]
fn reports_bcl_syntax_error_with_json_diagnostics_from_cli_command() {
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

    let output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "validate".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("bcl validate should return json diagnostics");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl validate output should be valid json");
    assert_eq!(payload["status"], "error");
    let first = &payload["rule_results"][0];
    assert_eq!(first["rule_id"], "BCL-SYNTAX-001");
    assert_eq!(first["error_id"], "bcl.syntax.parse_error");
    assert!(first["span"]["line"].as_u64().is_some());
    assert!(first["span"]["column"].as_u64().is_some());
}

#[test]
fn builds_bcl_package_with_workspace_block_dependencies_from_top_level_namespace() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let packages_root = temp_dir.path().join("packages");
    fs::create_dir_all(&packages_root).expect("packages root should be created");

    write_workspace_block_package(&packages_root, "dep.echo", "0.1.3");
    let package_root = write_bcl_package_consumer(
        &packages_root,
        "consumer.packaged_flow",
        "dep.echo",
        "^0.1.0",
    );

    let output = run(vec![
        "bcl".to_string(),
        "build".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", packages_root.display()),
        "--json".to_string(),
    ])
    .expect("package-aware bcl build should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl build output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["kind"], "bcl_build");
    assert_eq!(payload["package"]["id"], "consumer.packaged_flow");
    assert_eq!(payload["lowering_target"], "runtime-compat");
    assert!(
        payload["resolved_packages"]
            .as_array()
            .expect("resolved_packages should be an array")
            .iter()
            .any(|package| package["id"] == "dep.echo" && package["version"] == "0.1.3")
    );

    let artifact_path = payload["artifacts"][0]["path"]
        .as_str()
        .expect("artifact path should be present");
    assert!(std::path::Path::new(artifact_path).is_file());
    let emitted = fs::read_to_string(artifact_path).expect("artifact should be readable");
    assert!(emitted.contains("id: packaged_flow"));
    assert!(emitted.contains("dep.echo"));
}

#[test]
fn rejects_unknown_option_for_moc_bcl_validate_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let source_path = temp_dir.path().join("moc.bcl");
    fs::write(&source_path, "moc demo { name \"x\"; type frontend_lib; language tauri_ts; entry \"src/main.ts\"; uses { } depends_on_mocs { } protocols { } verification { command \"test\"; } accept \"ok\"; }")
        .expect("bcl should be written");

    let error = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "validate".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--bad".to_string(),
    ])
    .expect_err("unknown option should fail");

    assert!(error.contains("unknown option for moc bcl validate: --bad"));
}

#[test]
fn scaffolds_block_authoring_baseline_from_cli_commands() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");

    let init_output = run(vec![
        "block".to_string(),
        "init".to_string(),
        blocks_root.display().to_string(),
        "demo.slugify".to_string(),
        "--kind".to_string(),
        "rust".to_string(),
        "--target".to_string(),
        "shared".to_string(),
    ])
    .expect("block init should succeed");

    assert!(init_output.contains("scaffolded block:"));
    let block_root = blocks_root.join("demo.slugify");
    assert!(block_root.join("block.yaml").is_file());
    assert!(block_root.join("README.md").is_file());
    assert!(block_root.join("rust").join("lib.rs").is_file());
    assert!(block_root.join("rust").join("Cargo.toml").is_file());
    assert!(block_root.join("tests").is_dir());
    assert!(block_root.join("examples").is_dir());
    assert!(block_root.join("evaluators").is_dir());
    assert!(block_root.join("fixtures").is_dir());

    let fmt_output = run(vec![
        "block".to_string(),
        "fmt".to_string(),
        block_root.display().to_string(),
    ])
    .expect("block fmt should succeed");
    assert!(fmt_output.contains("formatted block contract:"));

    let check_output = run(vec![
        "block".to_string(),
        "check".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("block check should succeed");
    let payload: Value =
        serde_json::from_str(&check_output).expect("block check output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["block_id"], "demo.slugify");
    assert_eq!(payload["implementation"]["kind"], "rust");
    assert_eq!(payload["implementation"]["target"], "shared");
}

#[test]
fn scaffolds_moc_authoring_baseline_from_cli_commands() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let mocs_root = temp_dir.path().join("mocs");

    let init_output = run(vec![
        "moc".to_string(),
        "init".to_string(),
        mocs_root.display().to_string(),
        "hello-service".to_string(),
        "--type".to_string(),
        "backend_app".to_string(),
        "--backend-mode".to_string(),
        "service".to_string(),
        "--language".to_string(),
        "rust".to_string(),
    ])
    .expect("moc init should succeed");

    assert!(init_output.contains("scaffolded moc:"));
    let moc_root = mocs_root.join("hello-service");
    assert!(moc_root.join("moc.yaml").is_file());
    assert!(moc_root.join("README.md").is_file());
    assert!(
        moc_root
            .join("backend")
            .join("src")
            .join("main.rs")
            .is_file()
    );
    assert!(moc_root.join("backend").join("Cargo.toml").is_file());
    assert!(moc_root.join("input.example.json").is_file());
    assert!(moc_root.join("tests").is_dir());
    assert!(moc_root.join("examples").is_dir());

    let fmt_output = run(vec![
        "moc".to_string(),
        "fmt".to_string(),
        moc_root.display().to_string(),
    ])
    .expect("moc fmt should succeed");
    assert!(fmt_output.contains("formatted moc manifest:"));

    let check_output = run(vec![
        "moc".to_string(),
        "check".to_string(),
        blocks_root.display().to_string(),
        moc_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("moc check should succeed");
    let payload: Value =
        serde_json::from_str(&check_output).expect("moc check output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["moc_id"], "hello-service");
    assert_eq!(payload["moc_type"], "backend_app");
    assert_eq!(payload["backend_mode"], "service");
    assert_eq!(payload["descriptor_only"], true);
}

#[test]
fn scaffolds_moc_authoring_baseline_from_single_path_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let moc_root = temp_dir.path().join("mocs").join("hello-service");

    let init_output = run(vec![
        "moc".to_string(),
        "init".to_string(),
        moc_root.display().to_string(),
        "--type".to_string(),
        "backend_app".to_string(),
        "--backend-mode".to_string(),
        "service".to_string(),
        "--language".to_string(),
        "rust".to_string(),
    ])
    .expect("moc init should succeed from a single target path");

    assert!(init_output.contains("scaffolded moc:"));
    assert!(moc_root.join("moc.yaml").is_file());
    assert!(moc_root.join("README.md").is_file());
    assert!(moc_root.join("backend").join("Cargo.toml").is_file());

    let check_output = run(vec![
        "moc".to_string(),
        "check".to_string(),
        blocks_root.display().to_string(),
        moc_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("moc check should succeed");
    let payload: Value =
        serde_json::from_str(&check_output).expect("moc check output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["moc_id"], "hello-service");
}

#[test]
fn reports_missing_moc_entry_from_moc_check_json_output() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let mocs_root = temp_dir.path().join("mocs");

    run(vec![
        "moc".to_string(),
        "init".to_string(),
        mocs_root.display().to_string(),
        "broken-lib".to_string(),
        "--type".to_string(),
        "rust_lib".to_string(),
        "--language".to_string(),
        "rust".to_string(),
    ])
    .expect("moc init should succeed");

    let moc_root = mocs_root.join("broken-lib");
    fs::remove_file(moc_root.join("src").join("lib.rs")).expect("entry file should be removed");

    let error = run(vec![
        "moc".to_string(),
        "check".to_string(),
        blocks_root.display().to_string(),
        moc_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("moc check should report missing entry");
    let payload: Value =
        serde_json::from_str(&error).expect("moc check error output should be valid json");
    assert_eq!(payload["status"], "error");
    assert!(
        payload["errors"]
            .as_array()
            .expect("errors should be an array")
            .iter()
            .any(|item| item
                .as_str()
                .is_some_and(|message| message.contains("missing moc entry path")))
    );
}

#[test]
fn scaffolds_bcl_authoring_baseline_from_cli_commands() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let mocs_root = temp_dir.path().join("mocs");

    run(vec![
        "moc".to_string(),
        "init".to_string(),
        mocs_root.display().to_string(),
        "hello-bcl".to_string(),
        "--type".to_string(),
        "backend_app".to_string(),
        "--backend-mode".to_string(),
        "console".to_string(),
        "--language".to_string(),
        "rust".to_string(),
    ])
    .expect("moc init should succeed");

    let moc_root = mocs_root.join("hello-bcl");
    let init_output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "init".to_string(),
        moc_root.display().to_string(),
    ])
    .expect("moc bcl init should succeed");
    assert!(init_output.contains("scaffolded bcl:"));

    let bcl_path = moc_root.join("moc.bcl");
    assert!(bcl_path.is_file());
    let bcl_source = fs::read_to_string(&bcl_path).expect("bcl source should be readable");
    assert!(bcl_source.contains("moc hello-bcl"));
    assert!(bcl_source.contains("type backend_app(console);"));

    let fmt_output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "fmt".to_string(),
        moc_root.display().to_string(),
    ])
    .expect("moc bcl fmt should succeed");
    assert!(fmt_output.contains("formatted bcl source:"));

    let check_output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "check".to_string(),
        blocks_root.display().to_string(),
        moc_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("moc bcl check should succeed");
    let payload: Value =
        serde_json::from_str(&check_output).expect("moc bcl check output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["rule_results"], json!([]));
}

#[test]
fn runs_block_test_suite_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let block_root = write_evidence_block(temp_dir.path(), "demo.evidence");

    let output = run(vec![
        "block".to_string(),
        "test".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("block test should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("block test output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["suite"], "test");
    assert_eq!(payload["cases_run"], 2);
    assert_eq!(payload["evidence"]["tests_files"], 1);
    assert_eq!(payload["evidence"]["examples_files"], 2);
    assert_eq!(payload["evidence"]["fixtures_files"], 1);
}

#[test]
fn runs_block_eval_suite_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let block_root = write_evidence_block(temp_dir.path(), "demo.evidence");

    let output = run(vec![
        "block".to_string(),
        "eval".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("block eval should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("block eval output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["suite"], "eval");
    assert_eq!(payload["cases_run"], 1);
    assert_eq!(payload["evidence"]["evaluators_files"], 1);
    assert_eq!(payload["evidence"]["fixtures_files"], 1);
}

#[test]
fn runs_block_conformance_suite_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let block_root = write_evidence_block(temp_dir.path(), "demo.evidence");

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "block".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("block conformance should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("block conformance output should be valid json");
    assert_eq!(payload["suite"], "block");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["cases_run"], 3);
    assert!(
        payload["failures"]
            .as_array()
            .expect("failures should be an array")
            .is_empty()
    );
}

#[test]
fn runs_moc_conformance_suite_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let mocs_root = temp_dir.path().join("mocs");

    run(vec![
        "moc".to_string(),
        "init".to_string(),
        mocs_root.display().to_string(),
        "hello-service".to_string(),
        "--type".to_string(),
        "backend_app".to_string(),
        "--backend-mode".to_string(),
        "service".to_string(),
        "--language".to_string(),
        "rust".to_string(),
    ])
    .expect("moc init should succeed");

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "moc".to_string(),
        blocks_root.display().to_string(),
        mocs_root.join("hello-service").display().to_string(),
        "--json".to_string(),
    ])
    .expect("moc conformance should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("moc conformance output should be valid json");
    assert_eq!(payload["suite"], "moc");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["cases_run"], 1);
}

#[test]
fn runs_package_conformance_suite_for_third_party_consumer_workspace() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let adopter_root = temp_dir.path().join("third-party-adopter");
    let consumer_root = adopter_root.join("packages");
    let registry_root = adopter_root.join("file-registry");
    fs::create_dir_all(&consumer_root).expect("consumer root should be created");
    fs::create_dir_all(&registry_root).expect("registry root should be created");

    write_file_registry_package(&registry_root, "dep.shared", "0.2.4");
    let package_root =
        write_package_consumer(&consumer_root, "consumer.portal", "dep.shared", "^0.2.0");

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "package".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
        "--json".to_string(),
    ])
    .expect("package conformance should succeed for third-party consumer workspace");

    let payload: Value =
        serde_json::from_str(&output).expect("package conformance output should be valid json");
    assert_eq!(payload["suite"], "package");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["cases_run"], 3);
    assert!(
        payload["failures"]
            .as_array()
            .expect("failures should be an array")
            .is_empty()
    );
    assert!(package_root.join("blocks.lock").is_file());
}

#[test]
fn reports_runtime_host_capabilities_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.echo", "backend");

    let output = run(vec![
        "runtime".to_string(),
        "check".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("runtime check should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("runtime check output should be valid json");
    assert_eq!(payload["kind"], "runtime");
    assert!(
        matches!(payload["status"].as_str(), Some("ok" | "warn")),
        "runtime check should report a non-error status for rust backend blocks"
    );
    assert_eq!(
        payload["hosts"]
            .as_array()
            .expect("hosts should be an array")
            .len(),
        2
    );
}

#[test]
fn runtime_check_respects_explicit_host_selection() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.echo", "backend");

    let output = run(vec![
        "runtime".to_string(),
        "check".to_string(),
        block_root.display().to_string(),
        "--host".to_string(),
        "sync-cli".to_string(),
        "--json".to_string(),
    ])
    .expect("runtime check should succeed for explicit host selection");

    let payload: Value =
        serde_json::from_str(&output).expect("runtime check output should be valid json");
    let hosts = payload["hosts"]
        .as_array()
        .expect("hosts should be an array");
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0]["host_profile"], "sync-cli");
}

#[test]
fn runtime_check_renders_human_readable_output() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.echo", "backend");

    let output = run(vec![
        "runtime".to_string(),
        "check".to_string(),
        block_root.display().to_string(),
        "--host".to_string(),
        "sync-cli".to_string(),
    ])
    .expect("runtime check should succeed without json");

    assert!(output.contains("runtime check:"));
    assert!(output.contains("host sync-cli:"));
    assert!(!output.contains("\"hosts\""));
}

#[test]
fn runtime_check_reports_frontend_host_incompatibility_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.frontend_runtime", "frontend");

    let error = run(vec![
        "runtime".to_string(),
        "check".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("runtime check should fail for frontend targets");

    let payload: Value =
        serde_json::from_str(&error).expect("runtime check error should be valid json");
    assert_eq!(payload["kind"], "runtime");
    assert_eq!(payload["status"], "error");
    assert!(
        payload["hosts"]
            .as_array()
            .expect("hosts should be an array")
            .iter()
            .any(|host| host["errors"]
                .as_array()
                .expect("host errors should be an array")
                .iter()
                .any(|line| line
                    .as_str()
                    .is_some_and(|value| value.contains("does not support frontend targets"))))
    );
}

#[test]
fn runtime_check_rejects_unknown_host_values() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.echo", "backend");

    let error = run(vec![
        "runtime".to_string(),
        "check".to_string(),
        block_root.display().to_string(),
        "--host".to_string(),
        "invalid-host".to_string(),
    ])
    .expect_err("runtime check should reject unknown hosts");

    assert!(error.contains("unsupported runtime host profile"));
}

#[test]
fn runs_runtime_conformance_across_sync_and_tokio_hosts() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.echo", "backend");

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "runtime".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("runtime conformance should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("runtime conformance output should be valid json");
    assert_eq!(payload["suite"], "runtime");
    assert_eq!(payload["status"], "ok");
    assert!(
        payload["cases"]
            .as_array()
            .expect("cases should be an array")
            .iter()
            .any(|case| case["name"] == "runtime.output_parity" && case["status"] == "ok")
    );
}

#[test]
fn runtime_conformance_accepts_explicit_input_and_single_host() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.echo", "backend");
    fs::remove_dir_all(block_root.join("fixtures")).expect("fixtures dir should be removed");

    let input_path = temp_dir.path().join("explicit-runtime-input.json");
    fs::write(&input_path, r#"{ "text": "hello from explicit input" }"#)
        .expect("explicit runtime input should be written");

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "runtime".to_string(),
        block_root.display().to_string(),
        "--host".to_string(),
        "sync-cli".to_string(),
        "--input".to_string(),
        input_path.display().to_string(),
        "--json".to_string(),
    ])
    .expect("runtime conformance should succeed with explicit input");

    let payload: Value =
        serde_json::from_str(&output).expect("runtime conformance output should be valid json");
    let cases = payload["cases"]
        .as_array()
        .expect("cases should be an array");
    assert_eq!(payload["suite"], "runtime");
    assert_eq!(payload["status"], "ok");
    assert!(
        cases
            .iter()
            .any(|case| case["name"] == "runtime.check.sync-cli")
    );
    assert!(
        cases
            .iter()
            .any(|case| case["name"] == "runtime.execute.sync-cli")
    );
    assert!(
        !cases
            .iter()
            .any(|case| case["name"] == "runtime.output_parity")
    );
}

#[test]
fn runtime_conformance_fails_when_no_input_fixture_is_available() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_runtime_block(&blocks_root, "demo.echo", "backend");
    fs::remove_dir_all(block_root.join("fixtures")).expect("fixtures dir should be removed");

    let error = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "runtime".to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("runtime conformance should fail without fixtures or explicit input");

    assert!(error.contains("no runtime input fixture found"));
}

#[test]
fn runs_package_conformance_when_target_is_package_manifest_path() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let adopter_root = temp_dir.path().join("third-party-adopter");
    let consumer_root = adopter_root.join("packages");
    let registry_root = adopter_root.join("file-registry");
    fs::create_dir_all(&consumer_root).expect("consumer root should be created");
    fs::create_dir_all(&registry_root).expect("registry root should be created");

    write_file_registry_package(&registry_root, "dep.shared", "0.2.4");
    let package_root = write_package_consumer(
        &consumer_root,
        "consumer.manifest_path",
        "dep.shared",
        "^0.2.0",
    );

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "package".to_string(),
        package_root.join("package.yaml").display().to_string(),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
        "--json".to_string(),
    ])
    .expect("package conformance should accept package.yaml target paths");

    let payload: Value =
        serde_json::from_str(&output).expect("package conformance output should be valid json");
    assert_eq!(payload["suite"], "package");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["target"], package_root.display().to_string());
}

#[test]
fn package_conformance_reports_unsatisfied_dependency_as_json_failure() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let adopter_root = temp_dir.path().join("third-party-adopter");
    let consumer_root = adopter_root.join("packages");
    let registry_root = adopter_root.join("file-registry");
    fs::create_dir_all(&consumer_root).expect("consumer root should be created");
    fs::create_dir_all(&registry_root).expect("registry root should be created");

    let package_root = write_package_consumer(
        &consumer_root,
        "consumer.missing_dep",
        "dep.missing",
        "^0.2.0",
    );

    let error = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "package".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
        "--json".to_string(),
    ])
    .expect_err("package conformance should fail when dependency resolution is unsatisfied");

    let payload: Value =
        serde_json::from_str(&error).expect("package conformance error should be valid json");
    assert_eq!(payload["suite"], "package");
    assert_eq!(payload["status"], "error");
    assert!(
        payload["failures"]
            .as_array()
            .expect("failures should be an array")
            .iter()
            .any(|value| value["message"]
                .as_str()
                .is_some_and(|line| line.contains("no compatible release found"))),
        "package conformance should surface the underlying unsatisfied dependency failure"
    );
}

#[test]
fn package_conformance_returns_human_readable_output_without_json() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let adopter_root = temp_dir.path().join("third-party-adopter");
    let consumer_root = adopter_root.join("packages");
    let registry_root = adopter_root.join("file-registry");
    fs::create_dir_all(&consumer_root).expect("consumer root should be created");
    fs::create_dir_all(&registry_root).expect("registry root should be created");

    write_file_registry_package(&registry_root, "dep.shared", "0.2.4");
    let package_root = write_package_consumer(
        &consumer_root,
        "consumer.human_output",
        "dep.shared",
        "^0.2.0",
    );

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "package".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
    ])
    .expect("package conformance should succeed in human-readable mode");

    assert!(output.contains("conformance package: ok"));
    assert!(output.contains("case pkg.resolve: ok"));
    assert!(output.contains("case pkg.resolve.lock: ok"));
    assert!(output.contains("case pkg.resolve.repeat: ok"));
    assert!(
        !output.trim_start().starts_with('{'),
        "non-json mode should stay human-readable"
    );
}

#[test]
fn bcl_conformance_warn_mode_reports_parity_warning_without_failing() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let source_path = temp_dir.path().join("moc.bcl");
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
    .expect("bcl source should be written");

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: hello
name: Drifted Hello
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
acceptance_criteria:
  - works
"#,
    )
    .expect("manifest should be written");

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "bcl".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--check-against".to_string(),
        manifest_path.display().to_string(),
        "--gate-mode".to_string(),
        "warn".to_string(),
        "--json".to_string(),
    ])
    .expect("warn-mode bcl conformance should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl conformance output should be valid json");
    assert_eq!(payload["suite"], "bcl");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["gate_mode"], "warn");
    assert!(
        payload["warnings"]
            .as_array()
            .expect("warnings should be an array")
            .iter()
            .any(|value| value
                .as_str()
                .is_some_and(|message| message.contains("parity")))
    );
}

#[test]
fn bcl_conformance_error_mode_fails_on_parity_mismatch() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let source_path = temp_dir.path().join("moc.bcl");
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
    .expect("bcl source should be written");

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: hello
name: Drifted Hello
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
acceptance_criteria:
  - works
"#,
    )
    .expect("manifest should be written");

    let error = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "bcl".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--check-against".to_string(),
        manifest_path.display().to_string(),
        "--gate-mode".to_string(),
        "error".to_string(),
        "--json".to_string(),
    ])
    .expect_err("error-mode bcl conformance should fail");

    let payload: Value =
        serde_json::from_str(&error).expect("bcl conformance error should be valid json");
    assert_eq!(payload["suite"], "bcl");
    assert_eq!(payload["status"], "error");
    assert_eq!(payload["gate_mode"], "error");
    assert!(
        payload["failures"]
            .as_array()
            .expect("failures should be an array")
            .iter()
            .any(|value| value["case"] == "moc.bcl.parity")
    );
}

#[test]
fn bcl_conformance_off_mode_skips_parity_for_rollback() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let source_path = temp_dir.path().join("moc.bcl");
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
    .expect("bcl source should be written");

    let manifest_path = temp_dir.path().join("moc.yaml");
    fs::write(
        &manifest_path,
        r#"
id: hello
name: Drifted Hello
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
acceptance_criteria:
  - works
"#,
    )
    .expect("manifest should be written");

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "bcl".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--check-against".to_string(),
        manifest_path.display().to_string(),
        "--gate-mode".to_string(),
        "off".to_string(),
        "--json".to_string(),
    ])
    .expect("off-mode bcl conformance should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl conformance output should be valid json");
    assert_eq!(payload["suite"], "bcl");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["gate_mode"], "off");
    assert!(
        payload["cases"]
            .as_array()
            .expect("cases should be an array")
            .iter()
            .any(|value| value["name"] == "moc.bcl.parity" && value["status"] == "skipped")
    );
}

#[test]
fn runs_bcl_conformance_for_package_root_with_workspace_dependencies() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let packages_root = temp_dir.path().join("packages");
    fs::create_dir_all(&packages_root).expect("packages root should be created");

    write_workspace_block_package(&packages_root, "dep.echo", "0.1.3");
    let package_root = write_bcl_package_consumer(
        &packages_root,
        "consumer.packaged_flow",
        "dep.echo",
        "^0.1.0",
    );

    let output = run(vec![
        "conformance".to_string(),
        "run".to_string(),
        "bcl".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", packages_root.display()),
        "--gate-mode".to_string(),
        "off".to_string(),
        "--json".to_string(),
    ])
    .expect("package-root bcl conformance should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl conformance output should be valid json");
    assert_eq!(payload["suite"], "bcl");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["gate_mode"], "off");
    assert!(
        payload["cases"]
            .as_array()
            .expect("cases should be an array")
            .iter()
            .any(|case| case["name"] == "bcl.check" && case["status"] == "ok")
    );
    assert!(
        payload["cases"]
            .as_array()
            .expect("cases should be an array")
            .iter()
            .any(|case| case["name"] == "bcl.build" && case["status"] == "ok")
    );
    assert!(
        payload["cases"]
            .as_array()
            .expect("cases should be an array")
            .iter()
            .any(|case| case["name"] == "bcl.parity" && case["status"] == "skipped")
    );
    let artifact_path = payload["artifacts"][0]
        .as_str()
        .expect("artifact path should be present");
    assert!(std::path::Path::new(artifact_path).is_file());
}

#[test]
fn reports_bcl_plan_as_stable_json_from_cli_command() {
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

    let source_path = temp_dir.path().join("echo-plan.bcl");
    fs::write(
        &source_path,
        r#"
moc echo-plan {
  name "Echo Plan";
  type backend_app(console);
  language rust;
  entry "backend/src/main.rs";
  input {
    text: string required;
  }
  output {
    text: string required;
  }
  uses {
    block demo.echo;
  }
  depends_on_mocs { }
  protocols { }
  verification {
    command "cargo test";
    entry flow plan {
      step echo = demo.echo;
      bind input.text -> echo.text;
    }
  }
  accept "works";
}
"#,
    )
    .expect("bcl should be written");

    let output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "plan".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--json".to_string(),
    ])
    .expect("bcl plan should succeed");

    let payload: Value = serde_json::from_str(&output).expect("plan output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["moc_id"], "echo-plan");
    assert_eq!(payload["descriptor_only"], false);
    assert_eq!(payload["verification"]["entry_flow"], "plan");
    assert_eq!(payload["verification"]["plan"]["flow_id"], "plan");
    assert_eq!(payload["verification"]["plan"]["steps"][0]["id"], "echo");
    assert_eq!(
        payload["verification"]["plan"]["steps"][0]["block"],
        "demo.echo"
    );
}

#[test]
fn emits_bcl_and_checks_parity_against_manifest_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");

    let mocs_root = temp_dir.path().join("mocs");
    let local_dir = mocs_root.join("greeting-panel");
    fs::create_dir_all(&local_dir).expect("local dir should be created");
    let source_path = local_dir.join("moc.bcl");
    fs::write(
        &source_path,
        r#"
moc greeting-panel {
  name "Greeting Panel";
  type frontend_app;
  language tauri_ts;
  entry "src/main.ts";
  input { }
  output {
    mounted: boolean required;
  }
  uses { }
  depends_on_mocs {
    moc "greeting-api-service" via greeting-http;
  }
  protocols {
    protocol greeting-http {
      channel http;
      input { }
      output {
        title: string required;
        message: string required;
      }
    }
  }
  verification {
    command "node --test tests/greeting_panel.test.mjs";
  }
  accept "works";
}
"#,
    )
    .expect("bcl should be written");

    let manifest_path = temp_dir.path().join("greeting-panel.yaml");
    fs::write(
        &manifest_path,
        r#"
id: greeting-panel
name: Greeting Panel
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
  blocks: []
  internal_blocks: []
depends_on_mocs:
  - moc: greeting-api-service
    protocol: greeting-http
protocols:
  - name: greeting-http
    channel: http
    input_schema: {}
    output_schema:
      title:
        type: string
        required: true
      message:
        type: string
        required: true
verification:
  commands:
    - node --test tests/greeting_panel.test.mjs
acceptance_criteria:
  - works
"#,
    )
    .expect("manifest should be written");

    let remote_dir = mocs_root.join("greeting-api-service");
    fs::create_dir_all(&remote_dir).expect("remote dir should be created");
    fs::write(
        remote_dir.join("moc.yaml"),
        r#"
id: greeting-api-service
name: Greeting Api Service
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
      title:
        type: string
        required: true
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
    .expect("dependent moc should be written");

    let out_path = temp_dir.path().join("emitted.yaml");
    let output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "emit".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--out".to_string(),
        out_path.display().to_string(),
        "--check-against".to_string(),
        manifest_path.display().to_string(),
    ])
    .expect("emit should succeed");

    assert!(output.contains("emitted moc yaml:"));
    assert!(output.contains("parity: matched"));
    let emitted = fs::read_to_string(&out_path).expect("emitted yaml should exist");
    assert!(emitted.contains("id: greeting-panel"));
    assert!(emitted.contains("name: Greeting Panel"));
}

#[test]
fn exports_catalog_entries_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let _block_root = write_evidence_block(&blocks_root, "demo.evidence");

    let output = run(vec![
        "catalog".to_string(),
        "export".to_string(),
        blocks_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("catalog export should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("catalog export output should be valid json");
    let entries = payload
        .as_array()
        .expect("catalog export should return array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["id"], "demo.evidence");
    assert_eq!(entries[0]["implementation_kind"], "rust");
    assert_eq!(entries[0]["implementation_target"], "shared");
    assert_eq!(entries[0]["evidence"]["tests_files"], 1);
}

#[test]
fn searches_catalog_entries_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let _block_root = write_evidence_block(&blocks_root, "demo.evidence");

    let output = run(vec![
        "catalog".to_string(),
        "search".to_string(),
        blocks_root.display().to_string(),
        "demo".to_string(),
        "--json".to_string(),
    ])
    .expect("catalog search should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("catalog search output should be valid json");
    let entries = payload
        .as_array()
        .expect("catalog search should return array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["id"], "demo.evidence");
}

#[test]
fn runs_block_doctor_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    let block_root = write_evidence_block(&blocks_root, "demo.evidence");

    let output = run(vec![
        "block".to_string(),
        "doctor".to_string(),
        blocks_root.display().to_string(),
        block_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("block doctor should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("block doctor output should be valid json");
    assert_eq!(payload["target_kind"], "block");
    assert_eq!(payload["status"], "warn");
    assert!(payload["latest_diagnostic"].is_null());
    assert!(
        payload["recommendations"]
            .as_array()
            .expect("recommendations should be an array")
            .iter()
            .any(|value| value
                .as_str()
                .is_some_and(|message| message.contains("diagnostics")))
    );
}

#[test]
fn runs_moc_doctor_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let blocks_root = temp_dir.path().join("blocks");
    fs::create_dir_all(&blocks_root).expect("blocks root should be created");
    let mocs_root = temp_dir.path().join("mocs");

    run(vec![
        "moc".to_string(),
        "init".to_string(),
        mocs_root.display().to_string(),
        "hello-service".to_string(),
        "--type".to_string(),
        "backend_app".to_string(),
        "--backend-mode".to_string(),
        "console".to_string(),
        "--language".to_string(),
        "rust".to_string(),
    ])
    .expect("moc init should succeed");

    let output = run(vec![
        "moc".to_string(),
        "doctor".to_string(),
        blocks_root.display().to_string(),
        mocs_root.join("hello-service").display().to_string(),
        "--json".to_string(),
    ])
    .expect("moc doctor should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("moc doctor output should be valid json");
    assert_eq!(payload["target_kind"], "moc");
    assert_eq!(payload["launcher"]["kind"], "rust_backend");
    assert_eq!(payload["protocol_health"]["status"], "not_applicable");
    assert_eq!(payload["status"], "warn");
}

#[test]
fn runs_bcl_graph_with_json_output_from_cli_command() {
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
    let source_path = temp_dir.path().join("moc.bcl");
    fs::write(
        &source_path,
        r#"
moc hello {
  name "Hello";
  type backend_app(console);
  language rust;
  entry "backend/src/main.rs";
  input {
    text: string required;
  }
  output {
    text: string required;
  }
  uses {
    block demo.echo;
  }
  depends_on_mocs { }
  protocols { }
  verification {
    command "cargo test";
    entry flow plan {
      step echo = demo.echo;
      bind input.text -> echo.text;
    }
  }
  accept "works";
}
"#,
    )
    .expect("bcl source should be written");

    let output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "graph".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--json".to_string(),
    ])
    .expect("bcl graph should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl graph output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["moc_id"], "hello");
    assert!(
        payload["edges"]
            .as_array()
            .expect("edges should be an array")
            .iter()
            .any(|value| value["kind"] == "bind")
    );
}

#[test]
fn explains_bcl_failure_with_json_output_from_cli_command() {
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
    .expect("broken bcl should be written");

    let output = run(vec![
        "moc".to_string(),
        "bcl".to_string(),
        "explain".to_string(),
        blocks_root.display().to_string(),
        source_path.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("bcl explain should fail with structured report");

    let payload: Value =
        serde_json::from_str(&output).expect("bcl explain output should be valid json");
    assert_eq!(payload["status"], "error");
    assert_eq!(payload["phase"], "validate");
    assert!(
        payload["issues"]
            .as_array()
            .expect("issues should be an array")
            .iter()
            .any(|value| value["rule_id"] == "BCL-SYNTAX-001")
    );
}

#[test]
fn reports_block_compat_with_json_output_from_cli_command() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let before_root = temp_dir.path().join("before");
    let after_root = temp_dir.path().join("after");
    write_block(
        &before_root,
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
    write_block(
        &after_root,
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
  mode:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
    );

    let output = run(vec![
        "compat".to_string(),
        "block".to_string(),
        before_root.join("demo.echo").display().to_string(),
        after_root.join("demo.echo").display().to_string(),
        "--json".to_string(),
    ])
    .expect("block compat should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("block compat output should be valid json");
    assert_eq!(payload["target_kind"], "block");
    assert_eq!(payload["status"], "breaking");
    assert!(
        payload["changes"]
            .as_array()
            .expect("changes should be an array")
            .iter()
            .any(|value| value["path"] == "input_schema.mode")
    );
}

#[test]
fn previews_block_upgrade_with_json_output_from_cli_command() {
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

    let output = run(vec![
        "upgrade".to_string(),
        "block".to_string(),
        blocks_root.join("demo.echo").display().to_string(),
        "--json".to_string(),
    ])
    .expect("block upgrade preview should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("block upgrade output should be valid json");
    assert_eq!(payload["target_kind"], "block");
    assert_eq!(payload["status"], "preview");
    assert_eq!(payload["rule_set"], "r12-phase4-baseline");
    assert_eq!(
        payload["created_paths"]
            .as_array()
            .expect("created_paths should be an array")
            .len(),
        4
    );
    assert!(payload["preview"].as_str().is_some());
}
