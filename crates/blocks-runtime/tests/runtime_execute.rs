use std::cell::Cell;

use blocks_contract::BlockContract;
use blocks_runtime::{BlockExecutionError, BlockRunner, Runtime, RuntimeError};
use serde_json::{Value, json};

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
}

impl BlockRunner for StubRunner {
    fn run(&self, _block_id: &str, _input: &Value) -> Result<Value, BlockExecutionError> {
        self.calls.set(self.calls.get() + 1);
        self.output.clone()
    }
}

fn sample_contract() -> BlockContract {
    BlockContract::from_yaml_str(
        r#"
id: demo.echo
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
    .expect("contract should parse")
}

#[test]
fn rejects_invalid_input_before_runner_executes() {
    let runtime = Runtime::new();
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "text": "should not run" }));

    let result = runtime.execute(&contract, &json!({}), &runner);

    match result {
        Err(RuntimeError::InputValidationFailed { issues }) => {
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
    assert!(result.record.success);
    assert_eq!(runner.calls.get(), 1);
}

#[test]
fn rejects_invalid_output_after_runner_executes() {
    let runtime = Runtime::new();
    let contract = sample_contract();
    let runner = StubRunner::success(json!({ "unexpected": "value" }));

    let result = runtime.execute(&contract, &json!({ "text": "hello" }), &runner);

    match result {
        Err(RuntimeError::OutputValidationFailed { issues }) => {
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].path, "text");
        }
        other => panic!("unexpected result: {other:?}"),
    }
    assert_eq!(runner.calls.get(), 1);
}
