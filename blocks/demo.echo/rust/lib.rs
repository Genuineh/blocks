use blocks_runtime::BlockExecutionError;
use serde_json::{Value, json};

pub fn run(input: &Value) -> Result<Value, BlockExecutionError> {
    let text = input
        .get("text")
        .cloned()
        .ok_or_else(|| BlockExecutionError::new("missing string field: text"))?;

    Ok(json!({ "text": text }))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::run;

    #[test]
    fn echoes_text_payload() {
        let output = run(&json!({ "text": "hello world" })).expect("block should run");

        assert_eq!(output, json!({ "text": "hello world" }));
    }

    #[test]
    fn rejects_missing_text_field() {
        let error = run(&json!({})).expect_err("missing text should fail");

        assert!(error.to_string().contains("missing string field: text"));
    }

    #[test]
    fn quality_gate_fixture_echo_invariance() {
        let input: Value = serde_json::from_str(include_str!("../fixtures/success.input.json"))
            .expect("fixture input should parse");
        let expected: Value = serde_json::from_str(include_str!("../fixtures/success.output.json"))
            .expect("fixture output should parse");

        let output = run(&input).expect("fixture should execute");

        assert_eq!(output, expected);
    }
}
