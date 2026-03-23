use std::cell::Cell;
use std::fs;

use blocks_contract::BlockContract;
use blocks_runtime::{
    BlockExecutionError, BlockRunner, ExecutionContext, ExecutionEnvelope, Runtime, RuntimeError,
    RuntimeHost, SyncCliRuntimeHost, TokioServiceRuntimeHost, read_diagnostic_artifact,
    read_diagnostic_events,
};
use serde_json::{Value, json};
use tempfile::TempDir;

struct StubRunner {
    calls: Cell<usize>,
    output: Result<Value, BlockExecutionError>,
}

impl StubRunner {
    fn success(output: Value) -> Self {
        Self {
            calls: Cell::new(0),
            output: Ok(output),
        }
    }

    fn failure(message: &str) -> Self {
        Self {
            calls: Cell::new(0),
            output: Err(BlockExecutionError::new(message)),
        }
    }
}

impl BlockRunner for StubRunner {
    fn run(&self, _block_id: &str, _input: &Value) -> Result<Value, BlockExecutionError> {
        self.calls.set(self.calls.get() + 1);
        self.output.clone()
    }
}

fn sample_contract() -> BlockContract {
    BlockContract::from_yaml_str(&base_contract_yaml("")).expect("contract should parse")
}

fn contract_with_taxonomy(entries: &[&str]) -> BlockContract {
    let taxonomy = entries
        .iter()
        .map(|entry| format!("    - id: {entry}"))
        .collect::<Vec<_>>()
        .join("\n");
    let source = base_contract_yaml(&format!("errors:\n  taxonomy:\n{taxonomy}\n"));

    BlockContract::from_yaml_str(&source).expect("contract should parse")
}

fn active_contract_with_taxonomy(taxonomy: &[&str]) -> BlockContract {
    active_contract_with_observe(
        taxonomy,
        "observe:\n  metrics:\n    - execution_total\n  emits_failure_artifact: true\n  artifact_policy:\n    mode: on_failure\n",
    )
}

fn active_contract_with_observe(taxonomy: &[&str], observe_yaml: &str) -> BlockContract {
    let taxonomy_yaml = taxonomy
        .iter()
        .map(|id| format!("    - id: {id}"))
        .collect::<Vec<_>>()
        .join("\n");
    let source = base_contract_yaml(&format!(
        "debug:\n  enabled_in_dev: true\n  emits_structured_logs: true\n  log_fields:\n    - execution_id\n{observe_yaml}errors:\n  taxonomy:\n{taxonomy_yaml}\n"
    ))
    .replace("status: candidate", "status: active");
    BlockContract::from_yaml_str(&source).expect("contract should parse")
}

fn base_contract_yaml(extra: &str) -> String {
    format!(
        r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: candidate
owner: blocks-core-team
purpose: echo text
scope:
  - echo input text
non_goals:
  - persistence
inputs:
  - name: text
    description: text input
input_schema:
  text:
    type: string
    required: true
preconditions:
  - input exists
outputs:
  - name: text
    description: echoed text
output_schema:
  text:
    type: string
    required: true
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
{extra}
"#
    )
}

fn frontend_contract() -> BlockContract {
    BlockContract::from_yaml_str(
        &base_contract_yaml("").replace("target: shared", "target: frontend"),
    )
    .expect("frontend contract should parse")
}

#[test]
fn rejects_invalid_input_before_runner_executes() {
    let runtime = Runtime::new();
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "text": "should not run" }));

    let result = runtime.execute(&contract, &json!({}), &runner);

    match result {
        Err(RuntimeError::InputValidationFailed {
            execution_id,
            issues,
            ..
        }) => {
            assert!(!execution_id.is_empty());
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].path, "text");
        }
        other => panic!("unexpected result: {other:?}"),
    }
    assert_eq!(runner.calls.get(), 0);
}

#[test]
fn returns_execution_result_on_success() {
    let runtime = Runtime::new();
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "text": "hello" }));

    let result = runtime
        .execute(&contract, &json!({ "text": "hello" }), &runner)
        .expect("runtime should succeed");

    assert_eq!(result.output, json!({ "text": "hello" }));
    assert_eq!(result.record.block_id, "demo.echo");
    assert!(!result.record.execution_id.is_empty());
    assert_eq!(result.record.trace_id, None);
    assert!(result.record.success);
    assert_eq!(runner.calls.get(), 1);
}

#[test]
fn sync_and_tokio_hosts_execute_same_runtime_contract() {
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "text": "hello" }));
    let input = json!({ "text": "hello" });
    let context = ExecutionContext {
        trace_id: Some("trace-phase3".to_string()),
        moc_id: Some("moc.phase3".to_string()),
    };
    let sync_host = SyncCliRuntimeHost::with_runtime(Runtime::new());
    let tokio_host = TokioServiceRuntimeHost::with_runtime(Runtime::new());
    let envelope = ExecutionEnvelope {
        contract: &contract,
        input: &input,
        context: &context,
    };

    let sync_result = sync_host
        .execute_envelope(&envelope, &runner)
        .expect("sync host should execute the contract");
    let tokio_result = tokio_host
        .execute_envelope(&envelope, &runner)
        .expect("tokio host should execute the contract");

    assert_eq!(sync_result.output, json!({ "text": "hello" }));
    assert_eq!(tokio_result.output, json!({ "text": "hello" }));
    assert_eq!(sync_result.record.block_id, "demo.echo");
    assert_eq!(tokio_result.record.block_id, "demo.echo");
    assert!(sync_result.record.success);
    assert!(tokio_result.record.success);
    assert_eq!(runner.calls.get(), 2);
}

#[test]
fn runtime_hosts_report_incompatible_frontend_targets() {
    let contract = frontend_contract();
    let sync_report = SyncCliRuntimeHost::new().check_contract(&contract);
    let tokio_report = TokioServiceRuntimeHost::new().check_contract(&contract);

    assert_eq!(sync_report.status, "error");
    assert_eq!(tokio_report.status, "error");
    assert!(
        sync_report
            .errors
            .iter()
            .any(|line| line.contains("does not support frontend targets"))
    );
    assert!(
        tokio_report
            .errors
            .iter()
            .any(|line| line.contains("does not support frontend targets"))
    );
}

#[test]
fn rejects_invalid_output_after_runner_executes() {
    let runtime = Runtime::new();
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "unexpected": "value" }));

    let result = runtime.execute(&contract, &json!({ "text": "hello" }), &runner);

    match result {
        Err(RuntimeError::OutputValidationFailed {
            execution_id,
            issues,
            ..
        }) => {
            assert!(!execution_id.is_empty());
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].path, "text");
        }
        other => panic!("unexpected result: {other:?}"),
    }
    assert_eq!(runner.calls.get(), 1);
}

#[test]
fn redacts_sensitive_values_in_failure_and_points_to_diagnostic_artifact() {
    let runtime = Runtime::new();
    let contract = sample_contract();
    let runner = StubRunner::failure("authorization: Bearer top-secret-token");

    let error = runtime
        .execute(&contract, &json!({ "text": "hello" }), &runner)
        .expect_err("runtime should fail");
    let rendered = error.to_string();

    assert!(
        !rendered.contains("top-secret-token"),
        "failure output must not leak sensitive tokens"
    );
    assert!(
        rendered.contains("[REDACTED]"),
        "failure output should contain a redacted marker"
    );
    assert!(
        rendered.contains(".blocks/diagnostics"),
        "failure output should reference diagnostic artifact path"
    );
}

#[test]
fn writes_failure_artifact_with_basic_redaction() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "unexpected": "value" }));

    let result = runtime.execute(
        &contract,
        &json!({
            "text": "hello",
            "password": "unsafe",
            "Authorization": "Bearer abc123"
        }),
        &runner,
    );

    let execution_id = match result {
        Err(RuntimeError::OutputValidationFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };
    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed")
        .expect("artifact should exist");

    assert_eq!(
        artifact.input_snapshot["password"],
        Value::String("***REDACTED***".to_string())
    );
    assert_eq!(
        artifact.input_snapshot["Authorization"],
        Value::String("***REDACTED***".to_string())
    );
    assert!(artifact.output_snapshot.is_some());
    assert_eq!(artifact.error.error_id, "invalid_output");
    assert!(fs::metadata(runtime.diagnostics_root().join("events.jsonl")).is_ok());
}

#[test]
fn propagates_trace_id_into_events_for_contextual_execution() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "text": "hello" }));
    let context = ExecutionContext {
        trace_id: Some("trace-123".to_string()),
        moc_id: Some("moc.echo".to_string()),
    };

    let result = runtime
        .execute_with_context(&contract, &json!({ "text": "hello" }), &runner, &context)
        .expect("runtime should succeed");
    let events =
        read_diagnostic_events(runtime.diagnostics_root()).expect("events should be readable");
    let latest = events.last().expect("at least one event should exist");

    assert_eq!(result.record.trace_id.as_deref(), Some("trace-123"));
    assert_eq!(latest.trace_id.as_deref(), Some("trace-123"));
    assert_eq!(latest.moc_id.as_deref(), Some("moc.echo"));
}

#[test]
fn maps_runtime_error_to_matching_contract_taxonomy_id() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = contract_with_taxonomy(&["invalid_input", "internal_error"]);
    let runner = StubRunner::success(json!({ "text": "hello" }));

    let result = runtime.execute(&contract, &json!({}), &runner);
    let execution_id = match result {
        Err(RuntimeError::InputValidationFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };

    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed")
        .expect("artifact should exist");
    assert_eq!(artifact.error.error_id, "invalid_input");
}

#[test]
fn falls_back_to_controlled_runtime_error_id_when_output_taxonomy_mapping_missing() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = contract_with_taxonomy(&["dependency_unavailable"]);
    let runner = StubRunner::success(json!({ "unexpected": "value" }));

    let result = runtime.execute(&contract, &json!({ "text": "hello" }), &runner);
    let execution_id = match result {
        Err(RuntimeError::OutputValidationFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };

    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed")
        .expect("artifact should exist");
    assert_eq!(artifact.error.error_id, "runtime_fallback_invalid_output");
}

#[test]
fn keeps_taxonomy_error_id_when_runtime_failure_kind_exists() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = active_contract_with_taxonomy(&["invalid_input", "internal_error"]);
    let runner = StubRunner::failure("runner failed");

    let result = runtime.execute(&contract, &json!({ "text": "hello" }), &runner);

    let execution_id = match result {
        Err(RuntimeError::ExecutionFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };
    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed")
        .expect("artifact should exist");

    assert_eq!(artifact.error.error_id, "internal_error");
}

#[test]
fn falls_back_to_controlled_runtime_error_id_when_taxonomy_missing_preferred_kind() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = active_contract_with_taxonomy(&["invalid_input"]);
    let runner = StubRunner::failure("runner failed");

    let result = runtime.execute(&contract, &json!({ "text": "hello" }), &runner);

    let execution_id = match result {
        Err(RuntimeError::ExecutionFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };
    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed")
        .expect("artifact should exist");

    assert_eq!(
        artifact.error.error_id, "runtime_fallback_internal_error",
        "missing taxonomy mapping should use controlled fallback id"
    );
}

#[test]
fn skips_failure_artifact_when_observe_disables_it() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = active_contract_with_observe(
        &["invalid_input", "internal_error"],
        "observe:\n  metrics:\n    - execution_total\n  emits_failure_artifact: false\n  artifact_policy:\n    mode: on_failure\n",
    );
    let runner = StubRunner::failure("runner failed");

    let result = runtime.execute(&contract, &json!({ "text": "hello" }), &runner);

    let execution_id = match result {
        Err(RuntimeError::ExecutionFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };

    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed");
    assert!(
        artifact.is_none(),
        "artifact should be suppressed by observe"
    );

    let events =
        read_diagnostic_events(runtime.diagnostics_root()).expect("events should be readable");
    assert!(
        events.iter().any(|event| {
            event.execution_id == execution_id
                && event.event == "block.execution.failure"
                && event.error_id.as_deref() == Some("internal_error")
        }),
        "failure event should still be emitted even when artifact writing is disabled"
    );
}

#[test]
fn skips_failure_artifact_when_policy_mode_is_never() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = active_contract_with_observe(
        &["invalid_input", "internal_error"],
        "observe:\n  metrics:\n    - execution_total\n  emits_failure_artifact: true\n  artifact_policy:\n    mode: never\n",
    );
    let runner = StubRunner::failure("runner failed");

    let result = runtime.execute(&contract, &json!({ "text": "hello" }), &runner);

    let execution_id = match result {
        Err(RuntimeError::ExecutionFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };

    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed");
    assert!(
        artifact.is_none(),
        "artifact should be skipped when mode=never"
    );
}

#[test]
fn applies_failure_artifact_minimum_payload_policy() {
    let diagnostics = TempDir::new().expect("temp dir should be created");
    let runtime = Runtime::with_diagnostics_root(diagnostics.path().join(".blocks/diagnostics"));
    let contract = active_contract_with_observe(
        &["invalid_output", "internal_error"],
        "observe:\n  metrics:\n    - execution_total\n  emits_failure_artifact: true\n  artifact_policy:\n    mode: on_failure\n    on_failure_minimum:\n      include_input_snapshot: false\n      include_error_report: false\n      include_output_snapshot: never\n",
    );
    let runner = StubRunner::success(json!({ "unexpected": "value" }));

    let result = runtime.execute(
        &contract,
        &json!({
            "text": "hello",
            "password": "unsafe"
        }),
        &runner,
    );

    let execution_id = match result {
        Err(RuntimeError::OutputValidationFailed { execution_id, .. }) => execution_id,
        other => panic!("unexpected result: {other:?}"),
    };
    let artifact = read_diagnostic_artifact(runtime.diagnostics_root(), &execution_id)
        .expect("artifact lookup should succeed")
        .expect("artifact should exist");

    assert_eq!(artifact.input_snapshot, Value::Null);
    assert_eq!(artifact.output_snapshot, None);
    assert_eq!(artifact.error.error_id, "invalid_output");
    assert_eq!(artifact.error.message, "suppressed by artifact policy");
}
