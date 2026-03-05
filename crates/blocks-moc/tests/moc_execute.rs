use std::collections::BTreeMap;
use std::fs;

use blocks_moc::{MocComposer, MocError, MocManifest};
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
version: 0.1.0
status: candidate
owner: blocks-core-team
purpose: test echo block
scope:
  - echo input text
non_goals:
  - persistence
inputs:
  - name: text
    description: text input
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
preconditions:
  - input exists
outputs:
  - name: text
    description: echoed text
postconditions:
  - output exists
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
    when: input invalid
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
  - echoes text
"#,
    )
    .expect("contract should be written");
    fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
}

#[test]
fn validates_matching_moc_dependency_protocol() {
    let mocs_root = TempDir::new().expect("temp dir should be created");
    let provider_dir = mocs_root.path().join("hello-message-lib");
    let consumer_dir = mocs_root.path().join("hello-world-console");
    fs::create_dir_all(&provider_dir).expect("provider dir should be created");
    fs::create_dir_all(&consumer_dir).expect("consumer dir should be created");

    fs::write(
        provider_dir.join("moc.yaml"),
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
  - exposes a hello-message protocol
"#,
    )
    .expect("provider manifest should be written");

    let consumer = MocManifest::from_yaml_str(
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
  blocks: []
  internal_blocks: []
depends_on_mocs:
  - moc: hello-message-lib
    protocol: hello-message
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
  - uses the hello-message protocol
"#,
    )
    .expect("consumer manifest should parse");

    let result = consumer.validate_dependencies(mocs_root.path());

    assert!(result.is_ok());
}

#[test]
fn rejects_backend_app_without_backend_mode() {
    let result = MocManifest::from_yaml_str(
        r#"
id: invalid-backend
name: Invalid Backend
type: backend_app
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
  commands: []
acceptance_criteria: []
"#,
    );

    assert!(
        matches!(result, Err(MocError::InvalidDescriptor(message)) if message.contains("backend_mode"))
    );
}

#[test]
fn rejects_missing_internal_block_layout() {
    let moc_root = TempDir::new().expect("temp dir should be created");
    let manifest = MocManifest::from_yaml_str(
        r#"
id: missing-internal
name: Missing Internal
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks: []
  internal_blocks:
    - private.echo
depends_on_mocs: []
protocols: []
verification:
  commands: []
acceptance_criteria: []
"#,
    )
    .expect("manifest should parse");

    let result = manifest.validate_layout(moc_root.path());

    assert!(matches!(
        result,
        Err(MocError::InvalidDescriptor(message))
        if message.contains("missing internal block contract")
    ));
}

#[test]
fn rejects_moc_dependency_protocol_mismatch() {
    let mocs_root = TempDir::new().expect("temp dir should be created");
    let provider_dir = mocs_root.path().join("hello-message-lib");
    fs::create_dir_all(&provider_dir).expect("provider dir should be created");

    fs::write(
        provider_dir.join("moc.yaml"),
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
    channel: stdio
    input_schema: {}
    output_schema:
      text:
        type: string
        required: true
verification:
  commands:
    - cargo test
acceptance_criteria:
  - exposes a hello-message protocol
"#,
    )
    .expect("provider manifest should be written");

    let consumer = MocManifest::from_yaml_str(
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
  blocks: []
  internal_blocks: []
depends_on_mocs:
  - moc: hello-message-lib
    protocol: hello-message
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
  - uses the hello-message protocol
"#,
    )
    .expect("consumer manifest should parse");

    let result = consumer.validate_dependencies(mocs_root.path());

    assert!(matches!(
        result,
        Err(MocError::InvalidDescriptor(message))
        if message.contains("protocol mismatch")
    ));
}

#[test]
fn rejects_manifest_when_required_step_input_has_no_bind() {
    let blocks_root = TempDir::new().expect("temp dir should be created");
    write_demo_echo_block(&blocks_root);
    let registry = blocks_registry::Registry::load_from_root(blocks_root.path())
        .expect("registry should load");
    let manifest = MocManifest::from_yaml_str(
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
    .expect("manifest should parse");

    let result = MocComposer::new().plan(&manifest, &registry);

    assert!(matches!(
        result,
        Err(MocError::MissingBind {
            flow_id,
            step_id,
            field,
        })
        if flow_id == "plan" && step_id == "echo" && field == "text"
    ));
}

#[test]
fn rejects_manifest_when_bind_types_are_incompatible() {
    let blocks_root = TempDir::new().expect("temp dir should be created");
    write_demo_echo_block(&blocks_root);
    let registry = blocks_registry::Registry::load_from_root(blocks_root.path())
        .expect("registry should load");
    let manifest = MocManifest::from_yaml_str(
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
      type: number
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
  - reports type mismatch
"#,
    )
    .expect("manifest should parse");

    let result = MocComposer::new().plan(&manifest, &registry);

    assert!(matches!(
        result,
        Err(MocError::TypeMismatch {
            flow_id,
            step_id,
            bind_index,
            from,
            to,
            ..
        })
        if flow_id == "plan"
            && step_id == "echo"
            && bind_index == 1
            && from == "input.text"
            && to == "echo.text"
    ));
}

#[test]
fn rejects_manifest_when_bind_reference_is_invalid_with_context() {
    let blocks_root = TempDir::new().expect("temp dir should be created");
    write_demo_echo_block(&blocks_root);
    let registry = blocks_registry::Registry::load_from_root(blocks_root.path())
        .expect("registry should load");
    let manifest = MocManifest::from_yaml_str(
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
        - id: first
          block: demo.echo
        - id: second
          block: demo.echo
      binds:
        - from: second.text
          to: first.text
        - from: input.text
          to: second.text
acceptance_criteria:
  - reports invalid bind reference context
"#,
    )
    .expect("manifest should parse");

    let result = MocComposer::new().plan(&manifest, &registry);

    assert!(matches!(
        result,
        Err(MocError::InvalidReference {
            flow_id,
            step_id,
            bind_index,
            from,
            to,
            reference,
        })
        if flow_id == "plan"
            && step_id == "first"
            && bind_index == 1
            && from == "second.text"
            && to == "first.text"
            && reference == "second.text"
    ));
}

#[test]
fn builds_a_serial_execution_plan() {
    let blocks_root = TempDir::new().expect("temp dir should be created");
    write_demo_echo_block(&blocks_root);
    let registry = blocks_registry::Registry::load_from_root(blocks_root.path())
        .expect("registry should load");
    let manifest = MocManifest::from_yaml_str(
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
  - repeats hello through the second block
"#,
    )
    .expect("manifest should parse");

    let plan = MocComposer::new()
        .plan(&manifest, &registry)
        .expect("planner should succeed");

    assert_eq!(plan.last_step_id, "second");
    assert_eq!(plan.steps.len(), 2);
    assert_eq!(plan.moc_name, "echo-pipeline");

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

#[test]
fn rejects_flow_step_block_not_declared_in_uses_blocks() {
    let result = MocManifest::from_yaml_str(
        r#"
id: uses-mismatch
name: Uses Mismatch
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
  blocks: []
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
  - reports uses mismatch
"#,
    );

    assert!(matches!(
        result,
        Err(MocError::InvalidDescriptor(message))
        if message.contains("uses.blocks must exactly match")
    ));
}

#[test]
fn rejects_declared_uses_block_not_referenced_in_flows() {
    let result = MocManifest::from_yaml_str(
        r#"
id: declared-only
name: Declared Only
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
      steps: []
      binds: []
acceptance_criteria:
  - reports uses mismatch
"#,
    );

    assert!(matches!(
        result,
        Err(MocError::InvalidDescriptor(message))
        if message.contains("uses.blocks must exactly match")
    ));
}
