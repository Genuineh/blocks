use std::sync::{Mutex, OnceLock};

use blocks_contract::{
    ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV, BlockContract, ContractLoadError,
    ContractValidationConfig,
};
use serde_json::json;

fn env_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn full_contract() -> String {
    r#"
id: core.http.get
name: HTTP Get
version: 0.1.0
status: candidate
owner: blocks-core-team
purpose: Perform a minimal HTTP GET request and return status and body.
scope:
  - Execute one plain HTTP GET call and return response summary.
non_goals:
  - HTTPS/TLS requests.
inputs:
  - name: url
    description: Absolute plain HTTP endpoint URL.
input_schema:
  url:
    type: string
    required: true
    min_length: 5
preconditions:
  - URL uses http:// and is reachable from runtime network.
outputs:
  - name: status
    description: HTTP response status code.
output_schema:
  status:
    type: integer
    required: true
postconditions:
  - Response status is returned for successful call execution.
implementation:
  kind: rust
  entry: rust/lib.rs
  target: backend
dependencies:
  runtime:
    - std::net::TcpStream
side_effects:
  - Issues outbound network request.
timeouts:
  default_ms: 3000
resource_limits:
  memory_mb: 64
failure_modes:
  - id: invalid_input
    when: URL fails validation.
error_codes:
  - invalid_input
recovery_strategy:
  - Validate URL and scheme before invocation.
verification:
  automated:
    - cargo test -p blocks-runtime
evaluation:
  quality_gates:
    - Output schema validation passes.
acceptance_criteria:
  - Valid URL returns status field.
"#
    .to_string()
}

#[test]
fn rejects_invalid_yaml_contract() {
    let result = BlockContract::from_yaml_str("id: [");

    assert!(result.is_err());
}

#[test]
fn parses_implementation_metadata() {
    let contract = BlockContract::from_yaml_str(&full_contract()).expect("contract should parse");

    let implementation = contract
        .implementation
        .expect("implementation metadata should exist");
    assert_eq!(implementation.entry, "rust/lib.rs");
}

#[test]
fn rejects_missing_required_must_fields() {
    let source = full_contract().replace("owner: blocks-core-team\n", "");

    let result = BlockContract::from_yaml_str(&source);

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("owner field is required")
    ));
}

#[test]
fn rejects_empty_id() {
    let source = full_contract().replace("id: core.http.get", "id: \"\"");

    let result = BlockContract::from_yaml_str(&source);

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("id field is required")
    ));
}

#[test]
fn rejects_empty_required_list_fields() {
    let source = full_contract().replace(
        "scope:\n  - Execute one plain HTTP GET call and return response summary.\n",
        "scope: []\n",
    );

    let result = BlockContract::from_yaml_str(&source);

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("scope field must not be empty")
    ));
}

#[test]
fn rejects_invalid_tauri_target() {
    let result = BlockContract::from_yaml_str(
        &full_contract()
            .replace("kind: rust", "kind: tauri_ts")
            .replace("target: backend", "target: backend"),
    );

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("tauri_ts blocks must target frontend")
    ));
}

#[test]
fn rejects_invalid_taxonomy_id_pattern() {
    let source = format!(
        "{}errors:\n  taxonomy:\n    - id: Invalid-Input\n",
        full_contract()
    );

    let result = BlockContract::from_yaml_str(&source);

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("taxonomy id must match")
    ));
}

#[test]
fn rejects_duplicate_taxonomy_ids() {
    let source = format!(
        "{}errors:\n  taxonomy:\n    - id: invalid_input\n    - id: invalid_input\n",
        full_contract()
    );

    let result = BlockContract::from_yaml_str(&source);

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("duplicate taxonomy id: invalid_input")
    ));
}

#[test]
fn rejects_duplicate_failure_mode_ids() {
    let source = full_contract().replace(
        "failure_modes:\n  - id: invalid_input\n    when: URL fails validation.\n",
        "failure_modes:\n  - id: invalid_input\n    when: URL fails validation.\n  - id: invalid_input\n    when: duplicated\n",
    );

    let result = BlockContract::from_yaml_str(&source);

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("duplicate failure mode id: invalid_input")
    ));
}

#[test]
fn active_required_fields_warn_before_cutoff_date() {
    let _guard = env_guard().lock().expect("env lock");
    unsafe {
        std::env::remove_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV);
    }

    let source = full_contract().replace("status: candidate", "status: active");

    let (_, report) = BlockContract::from_yaml_str_with_report_and_config(
        &source,
        ContractValidationConfig {
            active_required_fields_enforcement: None,
            current_utc_date: Some((2026, 4, 15)),
        },
    )
    .expect("warn mode should not fail parsing");

    let warnings = report.warnings();
    assert!(warnings.iter().any(|item| item.path == "debug"));
    assert!(warnings.iter().any(|item| item.path == "observe"));
    assert!(warnings.iter().any(|item| item.path == "errors.taxonomy"));
}

#[test]
fn active_required_fields_error_on_and_after_cutoff_date() {
    let _guard = env_guard().lock().expect("env lock");
    unsafe {
        std::env::remove_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV);
    }

    let source = full_contract().replace("status: candidate", "status: active");

    let result = BlockContract::from_yaml_str_with_report_and_config(
        &source,
        ContractValidationConfig {
            active_required_fields_enforcement: None,
            current_utc_date: Some((2026, 4, 16)),
        },
    );

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("status is active")
    ));
}

#[test]
fn active_required_fields_enforcement_can_be_overridden_by_env() {
    let _guard = env_guard().lock().expect("env lock");
    unsafe {
        std::env::set_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV, "error");
    }

    let source = full_contract().replace("status: candidate", "status: active");
    let result = BlockContract::from_yaml_str_with_report(&source);

    unsafe {
        std::env::remove_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV);
    }

    assert!(matches!(
        result,
        Err(ContractLoadError::InvalidDefinition(message))
        if message.contains("status is active")
    ));
}

#[test]
fn reports_missing_required_fields() {
    let contract = BlockContract::from_yaml_str(&full_contract()).expect("contract should parse");

    let result = contract.validate_input(&json!({}));

    let issues = result.expect_err("validation should fail");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].path, "url");
}

#[test]
fn validates_minimal_string_input() {
    let contract = BlockContract::from_yaml_str(&full_contract()).expect("contract should parse");

    let result = contract.validate_input(&json!({
        "url": "https://example.com"
    }));

    assert!(result.is_ok());
}
