use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpanRange {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl SpanRange {
    pub fn new(line: usize, column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            line,
            column,
            end_line,
            end_column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleResult {
    pub error_id: String,
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub hint: Option<String>,
    pub span: SpanRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidateReport {
    pub status: String,
    pub source: String,
    pub rule_results: Vec<RuleResult>,
}

impl ValidateReport {
    pub fn ok(source: impl Into<String>) -> Self {
        Self {
            status: "ok".to_string(),
            source: source.into(),
            rule_results: Vec::new(),
        }
    }

    pub fn error(source: impl Into<String>, result: RuleResult) -> Self {
        Self {
            status: "error".to_string(),
            source: source.into(),
            rule_results: vec![result],
        }
    }
}
