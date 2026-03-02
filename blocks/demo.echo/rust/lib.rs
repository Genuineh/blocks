use blocks_runtime::BlockExecutionError;
use serde_json::{Value, json};

pub fn run(input: &Value) -> Result<Value, BlockExecutionError> {
    let text = input
        .get("text")
        .cloned()
        .ok_or_else(|| BlockExecutionError::new("missing string field: text"))?;

    Ok(json!({ "text": text }))
}
