use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const ACTIVE_BLOCK_REQUIRED_FIELDS_ENFORCEMENT: MigrationSeverity = MigrationSeverity::Warn;
pub const ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV: &str =
    "BLOCKS_ACTIVE_REQUIRED_FIELDS_ENFORCEMENT";
const ACTIVE_REQUIRED_FIELDS_ERROR_DATE_UTC: (i32, u8, u8) = (2026, 4, 16);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContract {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub non_goals: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<ContractItem>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<ContractItem>,
    #[serde(default)]
    pub postconditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<BlockImplementation>,
    #[serde(default)]
    pub dependencies: Value,
    #[serde(default)]
    pub side_effects: Vec<String>,
    #[serde(default)]
    pub timeouts: Value,
    #[serde(default)]
    pub resource_limits: Value,
    #[serde(default)]
    pub failure_modes: Vec<FailureMode>,
    #[serde(default)]
    pub error_codes: Vec<String>,
    #[serde(default)]
    pub recovery_strategy: Vec<String>,
    #[serde(default)]
    pub verification: Value,
    #[serde(default)]
    pub evaluation: Value,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observe: Option<ObserveContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<ErrorContract>,
    #[serde(default)]
    pub input_schema: BTreeMap<String, FieldSchema>,
    pub output_schema: BTreeMap<String, FieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractItem {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockImplementation {
    pub kind: ImplementationKind,
    pub entry: String,
    pub target: ImplementationTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugContract {
    #[serde(default)]
    pub enabled_in_dev: bool,
    #[serde(default)]
    pub emits_structured_logs: bool,
    #[serde(default)]
    pub log_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserveContract {
    #[serde(default)]
    pub metrics: Vec<String>,
    #[serde(default)]
    pub emits_failure_artifact: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_policy: Option<ArtifactPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPolicy {
    pub mode: ArtifactMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_failure_minimum: Option<FailureArtifactMinimum>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redaction_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<ArtifactRetention>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactMode {
    Always,
    OnFailure,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureArtifactMinimum {
    #[serde(default)]
    pub include_input_snapshot: bool,
    #[serde(default)]
    pub include_error_report: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_output_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRetention {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_days: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_total_mb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContract {
    #[serde(default)]
    pub taxonomy: Vec<TaxonomyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyEntry {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMode {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldSchema {
    #[serde(rename = "type")]
    pub field_type: ValueType,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(default, rename = "enum", skip_serializing_if = "Vec::is_empty")]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationKind {
    Rust,
    TauriTs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationTarget {
    Backend,
    Frontend,
    Shared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationSeverity {
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContractValidationConfig {
    pub active_required_fields_enforcement: Option<MigrationSeverity>,
    pub current_utc_date: Option<(i32, u8, u8)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractValidationIssue {
    pub path: String,
    pub message: String,
    pub severity: MigrationSeverity,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContractValidationReport {
    issues: Vec<ContractValidationIssue>,
}

impl ContractValidationReport {
    pub fn warnings(&self) -> Vec<ContractValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == MigrationSeverity::Warn)
            .cloned()
            .collect()
    }

    pub fn errors(&self) -> Vec<ContractValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == MigrationSeverity::Error)
            .cloned()
            .collect()
    }

    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == MigrationSeverity::Error)
    }

    fn push(
        &mut self,
        path: impl Into<String>,
        message: impl Into<String>,
        severity: MigrationSeverity,
    ) {
        self.issues.push(ContractValidationIssue {
            path: path.into(),
            message: message.into(),
            severity,
        });
    }

    fn format_errors(&self) -> String {
        self.errors()
            .into_iter()
            .map(|issue| format!("{}: {}", issue.path, issue.message))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

#[derive(Debug, Error)]
pub enum ContractLoadError {
    #[error("failed to parse block contract: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("invalid block contract definition: {0}")]
    InvalidDefinition(String),
}

impl BlockContract {
    pub fn from_yaml_str(source: &str) -> Result<Self, ContractLoadError> {
        let (contract, report) = Self::from_yaml_str_with_report_and_config(
            source,
            ContractValidationConfig::default(),
        )?;
        if report.has_errors() {
            return Err(ContractLoadError::InvalidDefinition(report.format_errors()));
        }
        Ok(contract)
    }

    pub fn from_yaml_str_with_report(
        source: &str,
    ) -> Result<(Self, ContractValidationReport), ContractLoadError> {
        Self::from_yaml_str_with_report_and_config(source, ContractValidationConfig::default())
    }

    pub fn from_yaml_str_with_report_and_config(
        source: &str,
        config: ContractValidationConfig,
    ) -> Result<(Self, ContractValidationReport), ContractLoadError> {
        let contract: Self = serde_yaml::from_str(source).map_err(ContractLoadError::from)?;
        let report = contract.validate_definition(&config);
        if report.has_errors() {
            return Err(ContractLoadError::InvalidDefinition(report.format_errors()));
        }
        Ok((contract, report))
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

    fn validate_definition(&self, config: &ContractValidationConfig) -> ContractValidationReport {
        let mut report = ContractValidationReport::default();

        self.validate_blocks_spec_required_fields(&mut report);

        if let Some(implementation) = &self.implementation {
            if implementation.entry.trim().is_empty() {
                report.push(
                    "implementation.entry",
                    "implementation.entry must not be empty",
                    MigrationSeverity::Error,
                );
            }

            if implementation.kind == ImplementationKind::TauriTs
                && implementation.target != ImplementationTarget::Frontend
            {
                report.push(
                    "implementation.target",
                    "tauri_ts blocks must target frontend",
                    MigrationSeverity::Error,
                );
            }
        }

        self.validate_taxonomy(&mut report);
        self.validate_failure_modes(&mut report);
        self.validate_active_required_fields(&mut report, config);

        report
    }

    fn validate_blocks_spec_required_fields(&self, report: &mut ContractValidationReport) {
        validate_required_string("id", Some(self.id.as_str()), report);
        validate_required_string("name", self.name.as_deref(), report);
        validate_required_string("version", self.version.as_deref(), report);
        validate_required_string("status", self.status.as_deref(), report);
        validate_required_string("owner", self.owner.as_deref(), report);
        validate_required_string("purpose", self.purpose.as_deref(), report);

        if self.implementation.is_none() {
            report.push(
                "implementation",
                "implementation field is required",
                MigrationSeverity::Error,
            );
        }

        validate_non_empty_string_list("scope", &self.scope, report);
        validate_non_empty_string_list("non_goals", &self.non_goals, report);
        validate_contract_items("inputs", &self.inputs, report);
        validate_non_empty_string_list("preconditions", &self.preconditions, report);
        validate_contract_items("outputs", &self.outputs, report);
        validate_non_empty_string_list("postconditions", &self.postconditions, report);
        validate_required_object("dependencies", &self.dependencies, report);
        validate_non_empty_string_list("side_effects", &self.side_effects, report);
        validate_required_object("timeouts", &self.timeouts, report);
        validate_required_object("resource_limits", &self.resource_limits, report);
        validate_non_empty_string_list("error_codes", &self.error_codes, report);
        validate_non_empty_string_list("recovery_strategy", &self.recovery_strategy, report);
        validate_required_object("verification", &self.verification, report);
        validate_required_object("evaluation", &self.evaluation, report);
        validate_non_empty_string_list("acceptance_criteria", &self.acceptance_criteria, report);

        if self.failure_modes.is_empty() {
            report.push(
                "failure_modes",
                "failure_modes field must not be empty",
                MigrationSeverity::Error,
            );
        }
    }

    fn validate_taxonomy(&self, report: &mut ContractValidationReport) {
        let Some(errors) = &self.errors else {
            return;
        };

        let mut seen = BTreeSet::new();
        for (index, taxonomy) in errors.taxonomy.iter().enumerate() {
            let path = format!("errors.taxonomy[{index}].id");
            let id = taxonomy.id.trim();
            if !is_valid_taxonomy_id(id) {
                report.push(
                    path.clone(),
                    "taxonomy id must match ^[a-z][a-z0-9_]{2,63}$",
                    MigrationSeverity::Error,
                );
            }

            if !seen.insert(id.to_string()) {
                report.push(
                    path,
                    format!("duplicate taxonomy id: {id}"),
                    MigrationSeverity::Error,
                );
            }
        }
    }

    fn validate_failure_modes(&self, report: &mut ContractValidationReport) {
        let mut seen = BTreeSet::new();
        for (index, failure_mode) in self.failure_modes.iter().enumerate() {
            let id = failure_mode.id.trim();
            if id.is_empty() {
                report.push(
                    format!("failure_modes[{index}].id"),
                    "failure mode id must not be empty",
                    MigrationSeverity::Error,
                );
                continue;
            }

            if !seen.insert(id.to_string()) {
                report.push(
                    format!("failure_modes[{index}].id"),
                    format!("duplicate failure mode id: {id}"),
                    MigrationSeverity::Error,
                );
            }
        }
    }

    fn validate_active_required_fields(
        &self,
        report: &mut ContractValidationReport,
        config: &ContractValidationConfig,
    ) {
        if !self.is_active() {
            return;
        }

        let enforcement = resolve_active_required_fields_enforcement(config);

        if self.debug.is_none() {
            report.push(
                "debug",
                "status is active, debug field is required",
                enforcement,
            );
        }

        if self.observe.is_none() {
            report.push(
                "observe",
                "status is active, observe field is required",
                enforcement,
            );
        }

        let has_taxonomy = self
            .errors
            .as_ref()
            .map(|errors| !errors.taxonomy.is_empty())
            .unwrap_or(false);
        if !has_taxonomy {
            report.push(
                "errors.taxonomy",
                "status is active, errors.taxonomy field is required",
                enforcement,
            );
        }
    }

    fn is_active(&self) -> bool {
        self.status
            .as_deref()
            .map(|status| status.trim().eq_ignore_ascii_case("active"))
            .unwrap_or(false)
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

fn is_valid_taxonomy_id(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_lowercase() {
        return false;
    }

    if !(3..=64).contains(&id.len()) {
        return false;
    }

    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn validate_required_string(
    path: &str,
    value: Option<&str>,
    report: &mut ContractValidationReport,
) {
    if value.map(|item| item.trim().is_empty()).unwrap_or(true) {
        report.push(
            path,
            format!("{path} field is required"),
            MigrationSeverity::Error,
        );
    }
}

fn validate_non_empty_string_list(
    path: &str,
    values: &[String],
    report: &mut ContractValidationReport,
) {
    if values.is_empty() {
        report.push(
            path,
            format!("{path} field must not be empty"),
            MigrationSeverity::Error,
        );
        return;
    }

    for (index, value) in values.iter().enumerate() {
        if value.trim().is_empty() {
            report.push(
                format!("{path}[{index}]"),
                "item must not be empty".to_string(),
                MigrationSeverity::Error,
            );
        }
    }
}

fn validate_contract_items(
    path: &str,
    values: &[ContractItem],
    report: &mut ContractValidationReport,
) {
    if values.is_empty() {
        report.push(
            path,
            format!("{path} field must not be empty"),
            MigrationSeverity::Error,
        );
        return;
    }

    for (index, value) in values.iter().enumerate() {
        if value.name.trim().is_empty() {
            report.push(
                format!("{path}[{index}].name"),
                "name must not be empty",
                MigrationSeverity::Error,
            );
        }
    }
}

fn validate_required_object(path: &str, value: &Value, report: &mut ContractValidationReport) {
    let is_valid = value.as_object().is_some_and(|object| !object.is_empty());
    if !is_valid {
        report.push(
            path,
            format!("{path} field must be a non-empty object"),
            MigrationSeverity::Error,
        );
    }
}

fn resolve_active_required_fields_enforcement(
    config: &ContractValidationConfig,
) -> MigrationSeverity {
    if let Some(severity) = config.active_required_fields_enforcement {
        return severity;
    }

    match env::var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "warn" => return MigrationSeverity::Warn,
            "error" => return MigrationSeverity::Error,
            "auto" => {}
            _ => {}
        },
        Err(_) => {}
    }

    let date = config.current_utc_date.unwrap_or_else(current_utc_date);
    if date >= ACTIVE_REQUIRED_FIELDS_ERROR_DATE_UTC {
        MigrationSeverity::Error
    } else {
        MigrationSeverity::Warn
    }
}

fn current_utc_date() -> (i32, u8, u8) {
    let days_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or(0);
    civil_from_days(days_since_epoch as i64)
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u8, u8) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year as i32, month as u8, day as u8)
}
