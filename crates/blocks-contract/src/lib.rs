use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContract {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub input_schema: BTreeMap<String, FieldSchema>,
    #[serde(default)]
    pub output_schema: BTreeMap<String, FieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    #[serde(rename = "type")]
    pub field_type: ValueType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub min_length: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default, rename = "enum")]
    pub allowed_values: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ContractLoadError {
    #[error("failed to parse block contract: {0}")]
    Parse(#[from] serde_yaml::Error),
}

impl BlockContract {
    pub fn from_yaml_str(source: &str) -> Result<Self, ContractLoadError> {
        serde_yaml::from_str(source).map_err(ContractLoadError::from)
    }

    pub fn validate_input(&self, input: &Value) -> Result<(), Vec<ValidationIssue>> {
        Self::validate_against_schema(&self.input_schema, input)
    }

    pub fn validate_output(&self, output: &Value) -> Result<(), Vec<ValidationIssue>> {
        Self::validate_against_schema(&self.output_schema, output)
    }

    fn validate_against_schema(
        schema: &BTreeMap<String, FieldSchema>,
        value: &Value,
    ) -> Result<(), Vec<ValidationIssue>> {
        if schema.is_empty() {
            return Ok(());
        }

        let Some(object) = value.as_object() else {
            return Err(vec![ValidationIssue {
                path: "$".to_string(),
                message: "value must be a JSON object".to_string(),
            }]);
        };

        let mut issues = Vec::new();

        for (field_name, field_schema) in schema {
            match object.get(field_name) {
                Some(current_value) => {
                    field_schema.validate(field_name, current_value, &mut issues)
                }
                None if field_schema.required => issues.push(ValidationIssue {
                    path: field_name.clone(),
                    message: "missing required field".to_string(),
                }),
                None => {}
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

impl FieldSchema {
    fn validate(&self, field_name: &str, value: &Value, issues: &mut Vec<ValidationIssue>) {
        if !self.field_type.matches(value) {
            issues.push(ValidationIssue {
                path: field_name.to_string(),
                message: format!("expected {}", self.field_type.as_str()),
            });
            return;
        }

        if let Some(actual) = value.as_str() {
            if let Some(min_length) = self.min_length {
                if actual.len() < min_length {
                    issues.push(ValidationIssue {
                        path: field_name.to_string(),
                        message: format!("must be at least {min_length} characters"),
                    });
                }
            }

            if let Some(max_length) = self.max_length {
                if actual.len() > max_length {
                    issues.push(ValidationIssue {
                        path: field_name.to_string(),
                        message: format!("must be at most {max_length} characters"),
                    });
                }
            }

            if !self.allowed_values.is_empty()
                && !self.allowed_values.iter().any(|item| item == actual)
            {
                issues.push(ValidationIssue {
                    path: field_name.to_string(),
                    message: "value is not in the allowed set".to_string(),
                });
            }
        }
    }
}

impl ValueType {
    fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Integer => "integer",
            Self::Boolean => "boolean",
            Self::Object => "object",
            Self::Array => "array",
        }
    }

    fn matches(self, value: &Value) -> bool {
        match self {
            Self::String => value.is_string(),
            Self::Number => value.is_number(),
            Self::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
            Self::Boolean => value.is_boolean(),
            Self::Object => value.is_object(),
            Self::Array => value.is_array(),
        }
    }
}
