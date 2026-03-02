use blocks_contract::BlockContract;
use serde_json::json;

fn sample_contract() -> &'static str {
    r#"
id: core.http.get
name: HTTP Get
input_schema:
  url:
    type: string
    required: true
    min_length: 5
"#
}

#[test]
fn rejects_invalid_yaml_contract() {
    let result = BlockContract::from_yaml_str("id: [");

    assert!(result.is_err());
}

#[test]
fn reports_missing_required_fields() {
    let contract = BlockContract::from_yaml_str(sample_contract()).expect("contract should parse");

    let result = contract.validate_input(&json!({}));

    let issues = result.expect_err("validation should fail");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].path, "url");
}

#[test]
fn validates_minimal_string_input() {
    let contract = BlockContract::from_yaml_str(sample_contract()).expect("contract should parse");

    let result = contract.validate_input(&json!({
        "url": "https://example.com"
    }));

    assert!(result.is_ok());
}
