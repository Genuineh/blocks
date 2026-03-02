use blocks_runtime::BlockExecutionError;
use serde_json::{Value, json};

pub fn run(input: &Value) -> Result<Value, BlockExecutionError> {
    let source = input
        .get("source")
        .cloned()
        .ok_or_else(|| BlockExecutionError::new("missing object field: source"))?;

    Ok(json!({ "result": source }))
}
