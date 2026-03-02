use std::collections::BTreeMap;

use blocks_contract::{FieldSchema, ValueType};
use blocks_registry::Registry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub name: String,
    pub entry: String,
    #[serde(default)]
    pub input_schema: BTreeMap<String, FieldSchema>,
    pub flows: Vec<Flow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub id: String,
    pub steps: Vec<FlowStep>,
    #[serde(default)]
    pub binds: Vec<Bind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub id: String,
    pub block: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bind {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub app_name: String,
    pub flow_id: String,
    pub last_step_id: String,
    pub steps: Vec<PlannedStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedStep {
    pub id: String,
    pub block: String,
    pub input_bindings: Vec<PlannedBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedBinding {
    pub from: String,
    pub to_field: String,
}

#[derive(Debug, Error)]
pub enum ComposeError {
    #[error("failed to parse app manifest: {0}")]
    ManifestParse(#[from] serde_yaml::Error),
    #[error("entry flow not found: {0}")]
    EntryFlowNotFound(String),
    #[error("entry flow has no steps: {0}")]
    EmptyFlow(String),
    #[error("unknown block: {0}")]
    UnknownBlock(String),
    #[error("missing bind for required field {step_id}.{field}")]
    MissingBind { step_id: String, field: String },
    #[error("invalid reference: {0}")]
    InvalidReference(String),
    #[error("type mismatch from {from} to {to}: expected {expected}, got {actual}")]
    TypeMismatch {
        from: String,
        to: String,
        expected: String,
        actual: String,
    },
}

#[derive(Debug, Default)]
pub struct Composer;

impl AppManifest {
    pub fn from_yaml_str(source: &str) -> Result<Self, ComposeError> {
        serde_yaml::from_str(source).map_err(ComposeError::from)
    }
}

impl Composer {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(
        &self,
        manifest: &AppManifest,
        registry: &Registry,
    ) -> Result<ExecutionPlan, ComposeError> {
        let flow = manifest
            .flows
            .iter()
            .find(|flow| flow.id == manifest.entry)
            .ok_or_else(|| ComposeError::EntryFlowNotFound(manifest.entry.clone()))?;

        if flow.steps.is_empty() {
            return Err(ComposeError::EmptyFlow(flow.id.clone()));
        }

        let steps = self.validate_and_collect_steps(manifest, flow, registry)?;
        let last_step_id = flow
            .steps
            .last()
            .expect("flow emptiness is checked before plan creation")
            .id
            .clone();

        Ok(ExecutionPlan {
            app_name: manifest.name.clone(),
            flow_id: flow.id.clone(),
            last_step_id,
            steps,
        })
    }

    fn validate_and_collect_steps(
        &self,
        manifest: &AppManifest,
        flow: &Flow,
        registry: &Registry,
    ) -> Result<Vec<PlannedStep>, ComposeError> {
        let mut planned_steps = Vec::with_capacity(flow.steps.len());

        for (step_index, step) in flow.steps.iter().enumerate() {
            let registered = registry
                .get(&step.block)
                .ok_or_else(|| ComposeError::UnknownBlock(step.block.clone()))?;
            let mut input_bindings = Vec::new();

            for bind in &flow.binds {
                let Some(target) = parse_target_ref(&bind.to) else {
                    return Err(ComposeError::InvalidReference(bind.to.clone()));
                };

                if target.step_id != step.id {
                    continue;
                }

                let Some(target_schema) = registered.contract.input_schema.get(&target.field)
                else {
                    return Err(ComposeError::InvalidReference(bind.to.clone()));
                };
                let source_type =
                    infer_source_type(manifest, flow, registry, step_index, &bind.from)?;

                if source_type != target_schema.field_type {
                    return Err(ComposeError::TypeMismatch {
                        from: bind.from.clone(),
                        to: bind.to.clone(),
                        expected: value_type_name(target_schema.field_type).to_string(),
                        actual: value_type_name(source_type).to_string(),
                    });
                }

                input_bindings.push(PlannedBinding {
                    from: bind.from.clone(),
                    to_field: target.field,
                });
            }

            for (field_name, schema) in &registered.contract.input_schema {
                if schema.required
                    && !input_bindings
                        .iter()
                        .any(|binding| binding.to_field == *field_name)
                {
                    return Err(ComposeError::MissingBind {
                        step_id: step.id.clone(),
                        field: field_name.clone(),
                    });
                }
            }

            planned_steps.push(PlannedStep {
                id: step.id.clone(),
                block: step.block.clone(),
                input_bindings,
            });
        }

        Ok(planned_steps)
    }
}

impl PlannedStep {
    pub fn build_input(
        &self,
        app_input: &Value,
        step_outputs: &BTreeMap<String, Value>,
    ) -> Result<Map<String, Value>, ComposeError> {
        let mut step_input = Map::new();

        for binding in &self.input_bindings {
            let value = resolve_value(app_input, step_outputs, &binding.from)?;
            step_input.insert(binding.to_field.clone(), value);
        }

        Ok(step_input)
    }
}

#[derive(Debug)]
struct TargetRef {
    step_id: String,
    field: String,
}

#[derive(Debug)]
enum SourceRef {
    Input { field: String },
    StepField { step_id: String, field: String },
}

fn parse_target_ref(source: &str) -> Option<TargetRef> {
    let (step_id, field) = source.split_once('.')?;
    if step_id.is_empty() || field.is_empty() {
        return None;
    }

    Some(TargetRef {
        step_id: step_id.to_string(),
        field: field.to_string(),
    })
}

fn parse_source_ref(source: &str) -> Option<SourceRef> {
    let (left, field) = source.split_once('.')?;
    if left.is_empty() || field.is_empty() {
        return None;
    }

    if left == "input" {
        Some(SourceRef::Input {
            field: field.to_string(),
        })
    } else {
        Some(SourceRef::StepField {
            step_id: left.to_string(),
            field: field.to_string(),
        })
    }
}

fn infer_source_type(
    manifest: &AppManifest,
    flow: &Flow,
    registry: &Registry,
    target_step_index: usize,
    reference: &str,
) -> Result<ValueType, ComposeError> {
    match parse_source_ref(reference)
        .ok_or_else(|| ComposeError::InvalidReference(reference.to_string()))?
    {
        SourceRef::Input { field } => manifest
            .input_schema
            .get(&field)
            .map(|schema| schema.field_type)
            .ok_or_else(|| ComposeError::InvalidReference(reference.to_string())),
        SourceRef::StepField { step_id, field } => {
            let source_index = flow
                .steps
                .iter()
                .position(|step| step.id == step_id)
                .ok_or_else(|| ComposeError::InvalidReference(reference.to_string()))?;

            if source_index >= target_step_index {
                return Err(ComposeError::InvalidReference(reference.to_string()));
            }

            let source_step = &flow.steps[source_index];
            let source_contract = &registry
                .get(&source_step.block)
                .ok_or_else(|| ComposeError::UnknownBlock(source_step.block.clone()))?
                .contract;

            source_contract
                .output_schema
                .get(&field)
                .map(|schema| schema.field_type)
                .ok_or_else(|| ComposeError::InvalidReference(reference.to_string()))
        }
    }
}

fn resolve_value(
    input: &Value,
    step_outputs: &BTreeMap<String, Value>,
    reference: &str,
) -> Result<Value, ComposeError> {
    match parse_source_ref(reference)
        .ok_or_else(|| ComposeError::InvalidReference(reference.to_string()))?
    {
        SourceRef::Input { field } => input
            .as_object()
            .and_then(|object| object.get(&field))
            .cloned()
            .ok_or_else(|| ComposeError::InvalidReference(reference.to_string())),
        SourceRef::StepField { step_id, field } => step_outputs
            .get(&step_id)
            .and_then(|value| value.as_object())
            .and_then(|object| object.get(&field))
            .cloned()
            .ok_or_else(|| ComposeError::InvalidReference(reference.to_string())),
    }
}

fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::String => "string",
        ValueType::Number => "number",
        ValueType::Integer => "integer",
        ValueType::Boolean => "boolean",
        ValueType::Object => "object",
        ValueType::Array => "array",
    }
}
