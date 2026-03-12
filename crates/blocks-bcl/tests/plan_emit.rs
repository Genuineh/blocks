use std::path::PathBuf;

use blocks_bcl::{check_against_file, emit_file};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root should exist")
        .parent()
        .expect("repo root should exist")
        .to_path_buf()
}

#[test]
fn emits_echo_pipeline_canonically_and_matches_existing_manifest() {
    let repo_root = repo_root();
    let blocks_root = repo_root.join("blocks");
    let bcl_path = repo_root.join("mocs/echo-pipeline/moc.bcl");
    let manifest_path = repo_root.join("mocs/echo-pipeline/moc.yaml");

    let first = emit_file(
        &blocks_root.display().to_string(),
        &bcl_path.display().to_string(),
    )
    .expect("emit should succeed");
    let second = emit_file(
        &blocks_root.display().to_string(),
        &bcl_path.display().to_string(),
    )
    .expect("emit should be deterministic");

    assert_eq!(first.yaml, second.yaml);
    check_against_file(&first.yaml, &manifest_path.display().to_string())
        .expect("echo-pipeline should satisfy parity");
}

#[test]
fn emits_greeting_panel_web_and_matches_existing_manifest() {
    let repo_root = repo_root();
    let blocks_root = repo_root.join("blocks");
    let bcl_path = repo_root.join("mocs/greeting-panel-web/moc.bcl");
    let manifest_path = repo_root.join("mocs/greeting-panel-web/moc.yaml");

    let emitted = emit_file(
        &blocks_root.display().to_string(),
        &bcl_path.display().to_string(),
    )
    .expect("emit should succeed");

    check_against_file(&emitted.yaml, &manifest_path.display().to_string())
        .expect("greeting-panel-web should satisfy parity");
}

#[test]
fn parity_check_fails_when_manifest_differs_after_canonicalization() {
    let emitted = r#"
id: demo
name: Demo
type: frontend_lib
language: tauri_ts
entry: src/lib.ts
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
acceptance_criteria:
  - works
"#;
    let mismatched = tempfile::NamedTempFile::new().expect("temp file should exist");
    std::fs::write(
        mismatched.path(),
        r#"
id: demo
name: Demo
type: frontend_lib
language: tauri_ts
entry: src/lib.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands: []
acceptance_criteria:
  - works
"#,
    )
    .expect("fixture should be written");

    let error = check_against_file(emitted, &mismatched.path().display().to_string())
        .expect_err("parity should fail");
    assert!(error.contains("does not match"));
}
