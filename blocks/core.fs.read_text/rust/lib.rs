use std::fs;

use blocks_runtime::BlockExecutionError;
use serde_json::{Value, json};

pub fn run(input: &Value) -> Result<Value, BlockExecutionError> {
    let path = input
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| BlockExecutionError::new("missing string field: path"))?;
    let text = fs::read_to_string(path).map_err(|error| {
        BlockExecutionError::new(format!("failed to read file {path}: {error}"))
    })?;

    Ok(json!({ "text": text }))
}
