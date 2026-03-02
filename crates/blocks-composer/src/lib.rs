use std::collections::BTreeMap;

use blocks_contract::{FieldSchema, ValueType};
use blocks_registry::Registry;
use blocks_runtime::{BlockRunner, Runtime, RuntimeError};
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

#[derive(Debug, Clone, PartialEq)]
pub struct ComposeResult {
    pub last_step_id: String,
    pub output: Value,
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
    #[error("runtime failed for step {step_id}: {source}")]
    Runtime {
        step_id: String,
        #[source]
        source: RuntimeError,
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

    pub fn execute(
        &self,
        manifest: &AppManifest,
        input: &Value,
        registry: &Registry,
        runner: &impl BlockRunner,
    ) -> Result<ComposeResult, ComposeError> {
        let flow = manifest
            .flows
            .iter()
            .find(|flow| flow.id == manifest.entry)
            .ok_or_else(|| ComposeError::EntryFlowNotFound(manifest.entry.clone()))?;

        if flow.steps.is_empty() {
            return Err(ComposeError::EmptyFlow(flow.id.clone()));
        }

        self.validate_flow(manifest, flow, registry)?;

        let runtime = Runtime::new();
        let mut step_outputs: BTreeMap<String, Value> = BTreeMap::new();

        for step in &flow.steps {
            let step_input = self.build_step_input(manifest, flow, input, &step_outputs, step)?;
            let contract = &registry
                .get(&step.block)
                .ok_or_else(|| ComposeError::UnknownBlock(step.block.clone()))?
                .contract;

            let result = runtime
                .execute(contract, &Value::Object(step_input), runner)
                .map_err(|source| ComposeError::Runtime {
                    step_id: step.id.clone(),
                    source,
                })?;

            step_outputs.insert(step.id.clone(), result.output);
        }

        let last_step = flow
            .steps
            .last()
            .expect("flow emptiness is checked before execution");

        Ok(ComposeResult {
            last_step_id: last_step.id.clone(),
            output: step_outputs
                .remove(&last_step.id)
                .expect("last step output should exist"),
        })
    }

    fn validate_flow(
        &self,
        manifest: &AppManifest,
        flow: &Flow,
        registry: &Registry,
    ) -> Result<(), ComposeError> {
        for (step_index, step) in flow.steps.iter().enumerate() {
            let registered = registry
                .get(&step.block)
                .ok_or_else(|| ComposeError::UnknownBlock(step.block.clone()))?;

            for (field_name, schema) in &registered.contract.input_schema {
                if !schema.required {
                    continue;
                }

                let Some(bind) = flow.binds.iter().find(|bind| {
                    matches!(
                        parse_target_ref(&bind.to),
                        Some(TargetRef { step_id, field })
                            if step_id == step.id && field == *field_name
                    )
                }) else {
                    return Err(ComposeError::MissingBind {
                        step_id: step.id.clone(),
                        field: field_name.clone(),
                    });
                };

                let source_type =
                    infer_source_type(manifest, flow, registry, step_index, &bind.from)?;
                let target_type = schema.field_type;

                if source_type != target_type {
                    return Err(ComposeError::TypeMismatch {
                        from: bind.from.clone(),
                        to: bind.to.clone(),
                        expected: value_type_name(target_type).to_string(),
                        actual: value_type_name(source_type).to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn build_step_input(
        &self,
        _manifest: &AppManifest,
        flow: &Flow,
        input: &Value,
        step_outputs: &BTreeMap<String, Value>,
        step: &FlowStep,
    ) -> Result<Map<String, Value>, ComposeError> {
        let mut step_input = Map::new();

        for bind in &flow.binds {
            let Some(target) = parse_target_ref(&bind.to) else {
                return Err(ComposeError::InvalidReference(bind.to.clone()));
            };

            if target.step_id != step.id {
                continue;
            }

            let value = resolve_value(input, step_outputs, &bind.from)?;
            step_input.insert(target.field, value);
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
