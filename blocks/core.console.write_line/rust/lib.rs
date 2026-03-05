use blocks_runtime::BlockExecutionError;
use serde_json::{Value, json};

pub fn run(input: &Value) -> Result<Value, BlockExecutionError> {
    let text = input
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| BlockExecutionError::new("missing string field: text"))?;

    println!("{text}");

    Ok(json!({ "written": true }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::run;

    #[test]
    fn writes_a_line() {
        let output = run(&json!({ "text": "hello world" })).expect("block should run");

        assert_eq!(output, json!({ "written": true }));
    }
}
