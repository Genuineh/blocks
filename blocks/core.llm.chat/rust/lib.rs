use blocks_runtime::BlockExecutionError;
use serde_json::{Value, json};

pub fn run(input: &Value) -> Result<Value, BlockExecutionError> {
    let prompt = input
        .get("prompt")
        .and_then(Value::as_str)
        .ok_or_else(|| BlockExecutionError::new("missing string field: prompt"))?;

    Ok(json!({ "text": prompt }))
}
