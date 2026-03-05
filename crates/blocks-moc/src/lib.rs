use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

use blocks_contract::{BlockContract, FieldSchema, ValueType};
use blocks_registry::Registry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MocManifest {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub moc_type: MocType,
    #[serde(default)]
    pub backend_mode: Option<BackendMode>,
    pub language: String,
    pub entry: String,
    #[serde(default)]
    pub public_contract: MocContract,
    #[serde(default)]
    pub uses: MocUses,
    #[serde(default)]
    pub depends_on_mocs: Vec<MocDependency>,
    #[serde(default)]
    pub protocols: Vec<MocProtocol>,
    #[serde(default)]
    pub verification: MocVerification,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MocType {
    RustLib,
    FrontendLib,
    FrontendApp,
    BackendApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendMode {
    Console,
    Service,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MocContract {
    #[serde(default)]
    pub input_schema: BTreeMap<String, FieldSchema>,
    #[serde(default)]
    pub output_schema: BTreeMap<String, FieldSchema>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MocUses {
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub internal_blocks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MocDependency {
    pub moc: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MocProtocol {
    pub name: String,
    pub channel: String,
    #[serde(default)]
    pub input_schema: BTreeMap<String, FieldSchema>,
    #[serde(default)]
    pub output_schema: BTreeMap<String, FieldSchema>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MocVerification {
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub entry_flow: Option<String>,
    #[serde(default)]
    pub flows: Vec<Flow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub moc_name: String,
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
    pub flow_id: String,
    pub step_id: String,
    pub bind_index: usize,
    pub from: String,
    pub to: String,
    pub to_field: String,
}

#[derive(Debug, Error)]
pub enum MocError {
    #[error("failed to parse moc manifest: {0}")]
    ManifestParse(#[from] serde_yaml::Error),
    #[error("entry flow not found: {0}")]
    EntryFlowNotFound(String),
    #[error("validation flow is not configured")]
    ValidationFlowNotConfigured,
    #[error("entry flow has no steps: {0}")]
    EmptyFlow(String),
    #[error("unknown block: {0}")]
    UnknownBlock(String),
    #[error("missing bind for required field {step_id}.{field} in flow {flow_id}")]
    MissingBind {
        flow_id: String,
        step_id: String,
        field: String,
    },
    #[error("invalid reference in flow {flow_id} step {step_id} bind #{bind_index}: {reference}")]
    InvalidReference {
        flow_id: String,
        step_id: String,
        bind_index: usize,
        from: String,
        to: String,
        reference: String,
    },
    #[error(
        "type mismatch in flow {flow_id} step {step_id} bind #{bind_index} from {from} to {to}: expected {expected}, got {actual}"
    )]
    TypeMismatch {
        flow_id: String,
        step_id: String,
        bind_index: usize,
        from: String,
        to: String,
        expected: String,
        actual: String,
    },
    #[error("invalid moc descriptor: {0}")]
    InvalidDescriptor(String),
}

#[derive(Debug, Default)]
pub struct MocComposer;

impl MocManifest {
    pub fn from_yaml_str(source: &str) -> Result<Self, MocError> {
        let manifest: Self = serde_yaml::from_str(source).map_err(MocError::from)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), MocError> {
        if self.id.trim().is_empty() {
            return Err(MocError::InvalidDescriptor(
                "id must not be empty".to_string(),
            ));
        }

        if self.name.trim().is_empty() {
            return Err(MocError::InvalidDescriptor(
                "name must not be empty".to_string(),
            ));
        }

        if self.language.trim().is_empty() {
            return Err(MocError::InvalidDescriptor(
                "language must not be empty".to_string(),
            ));
        }

        if self.entry.trim().is_empty() {
            return Err(MocError::InvalidDescriptor(
                "entry must not be empty".to_string(),
            ));
        }

        match self.moc_type {
            MocType::BackendApp if self.backend_mode.is_none() => {
                return Err(MocError::InvalidDescriptor(
                    "backend_mode is required when type=backend_app".to_string(),
                ));
            }
            MocType::BackendApp => {}
            _ if self.backend_mode.is_some() => {
                return Err(MocError::InvalidDescriptor(
                    "backend_mode is allowed only when type=backend_app".to_string(),
                ));
            }
            _ => {}
        }

        let mut protocol_names = std::collections::BTreeSet::new();
        for protocol in &self.protocols {
            if protocol.name.trim().is_empty() {
                return Err(MocError::InvalidDescriptor(
                    "protocol.name must not be empty".to_string(),
                ));
            }
            if protocol.channel.trim().is_empty() {
                return Err(MocError::InvalidDescriptor(
                    "protocol.channel must not be empty".to_string(),
                ));
            }
            if !protocol_names.insert(protocol.name.as_str()) {
                return Err(MocError::InvalidDescriptor(format!(
                    "duplicate protocol name: {}",
                    protocol.name
                )));
            }
        }

        for dependency in &self.depends_on_mocs {
            if dependency.moc.trim().is_empty() {
                return Err(MocError::InvalidDescriptor(
                    "depends_on_mocs[].moc must not be empty".to_string(),
                ));
            }
            if dependency.protocol.trim().is_empty() {
                return Err(MocError::InvalidDescriptor(
                    "depends_on_mocs[].protocol must not be empty".to_string(),
                ));
            }
        }

        if self.verification.entry_flow.is_some() && self.verification.flows.is_empty() {
            return Err(MocError::InvalidDescriptor(
                "verification.flows is required when verification.entry_flow is set".to_string(),
            ));
        }

        if self.verification.entry_flow.is_none() && !self.verification.flows.is_empty() {
            return Err(MocError::InvalidDescriptor(
                "verification.entry_flow is required when verification.flows is set".to_string(),
            ));
        }

        self.validate_uses_blocks_flow_consistency()?;

        Ok(())
    }

    pub fn has_validation_flow(&self) -> bool {
        self.verification.entry_flow.is_some()
    }

    pub fn validate_layout(&self, moc_root: &Path) -> Result<(), MocError> {
        for block_id in &self.uses.internal_blocks {
            let block_root = moc_root.join("internal_blocks").join(block_id);
            let contract_path = block_root.join("block.yaml");

            if !contract_path.is_file() {
                return Err(MocError::InvalidDescriptor(format!(
                    "missing internal block contract: {}",
                    contract_path.display()
                )));
            }

            let contract_source = fs::read_to_string(&contract_path).map_err(|error| {
                MocError::InvalidDescriptor(format!(
                    "failed to read internal block contract {}: {error}",
                    contract_path.display()
                ))
            })?;
            let contract = BlockContract::from_yaml_str(&contract_source).map_err(|error| {
                MocError::InvalidDescriptor(format!(
                    "failed to load internal block contract {}: {error}",
                    contract_path.display()
                ))
            })?;

            if contract.id != *block_id {
                return Err(MocError::InvalidDescriptor(format!(
                    "internal block id mismatch: expected {}, got {}",
                    block_id, contract.id
                )));
            }

            if let Some(implementation) = contract.implementation {
                let implementation_path = block_root.join(implementation.entry);
                if !implementation_path.is_file() {
                    return Err(MocError::InvalidDescriptor(format!(
                        "missing internal block implementation: {}",
                        implementation_path.display()
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn validate_dependencies(&self, mocs_root: &Path) -> Result<(), MocError> {
        for dependency in &self.depends_on_mocs {
            let dependency_manifest_path = mocs_root.join(&dependency.moc).join("moc.yaml");
            if !dependency_manifest_path.is_file() {
                return Err(MocError::InvalidDescriptor(format!(
                    "missing dependent moc manifest: {}",
                    dependency_manifest_path.display()
                )));
            }

            let dependency_source =
                fs::read_to_string(&dependency_manifest_path).map_err(|error| {
                    MocError::InvalidDescriptor(format!(
                        "failed to read dependent moc manifest {}: {error}",
                        dependency_manifest_path.display()
                    ))
                })?;
            let dependency_manifest = Self::from_yaml_str(&dependency_source)?;

            let local_protocol = self
                .protocols
                .iter()
                .find(|protocol| protocol.name == dependency.protocol)
                .ok_or_else(|| {
                    MocError::InvalidDescriptor(format!(
                        "local protocol not declared for dependency {}: {}",
                        dependency.moc, dependency.protocol
                    ))
                })?;
            let remote_protocol = dependency_manifest
                .protocols
                .iter()
                .find(|protocol| protocol.name == dependency.protocol)
                .ok_or_else(|| {
                    MocError::InvalidDescriptor(format!(
                        "dependent moc {} does not declare protocol {}",
                        dependency.moc, dependency.protocol
                    ))
                })?;

            if local_protocol.channel != remote_protocol.channel
                || local_protocol.input_schema != remote_protocol.input_schema
                || local_protocol.output_schema != remote_protocol.output_schema
            {
                return Err(MocError::InvalidDescriptor(format!(
                    "protocol mismatch for dependency {}: {}",
                    dependency.moc, dependency.protocol
                )));
            }
        }

        Ok(())
    }

    fn validate_uses_blocks_flow_consistency(&self) -> Result<(), MocError> {
        if self.verification.flows.is_empty() {
            return Ok(());
        }

        let internal_blocks: BTreeSet<&str> = self
            .uses
            .internal_blocks
            .iter()
            .map(String::as_str)
            .collect();
        let mut flow_blocks: BTreeSet<&str> = BTreeSet::new();
        for flow in &self.verification.flows {
            for step in &flow.steps {
                if !internal_blocks.contains(step.block.as_str()) {
                    flow_blocks.insert(step.block.as_str());
                }
            }
        }

        let declared_blocks: BTreeSet<&str> = self.uses.blocks.iter().map(String::as_str).collect();
        if flow_blocks == declared_blocks {
            return Ok(());
        }

        let declared_only: Vec<String> = declared_blocks
            .difference(&flow_blocks)
            .map(|item| (*item).to_string())
            .collect();
        let flow_only: Vec<String> = flow_blocks
            .difference(&declared_blocks)
            .map(|item| (*item).to_string())
            .collect();
        Err(MocError::InvalidDescriptor(format!(
            "uses.blocks must exactly match external flow step blocks; declared_only={declared_only:?}, flow_only={flow_only:?}"
        )))
    }
}

impl MocComposer {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(
        &self,
        manifest: &MocManifest,
        registry: &Registry,
    ) -> Result<ExecutionPlan, MocError> {
        manifest.validate()?;
        let entry_flow = manifest
            .verification
            .entry_flow
            .as_deref()
            .ok_or(MocError::ValidationFlowNotConfigured)?;
        let flow = manifest
            .verification
            .flows
            .iter()
            .find(|flow| flow.id == entry_flow)
            .ok_or_else(|| MocError::EntryFlowNotFound(entry_flow.to_string()))?;

        if flow.steps.is_empty() {
            return Err(MocError::EmptyFlow(flow.id.clone()));
        }

        let steps = self.validate_and_collect_steps(manifest, flow, registry)?;
        let last_step_id = flow
            .steps
            .last()
            .expect("flow emptiness is checked before plan creation")
            .id
            .clone();

        Ok(ExecutionPlan {
            moc_name: manifest.id.clone(),
            flow_id: flow.id.clone(),
            last_step_id,
            steps,
        })
    }

    fn validate_and_collect_steps(
        &self,
        manifest: &MocManifest,
        flow: &Flow,
        registry: &Registry,
    ) -> Result<Vec<PlannedStep>, MocError> {
        let mut planned_steps = Vec::with_capacity(flow.steps.len());

        for (step_index, step) in flow.steps.iter().enumerate() {
            let registered = registry
                .get(&step.block)
                .ok_or_else(|| MocError::UnknownBlock(step.block.clone()))?;
            let mut input_bindings = Vec::new();

            for (bind_offset, bind) in flow.binds.iter().enumerate() {
                let bind_index = bind_offset + 1;
                let Some(target) = parse_target_ref(&bind.to) else {
                    return Err(invalid_reference_for_bind(
                        flow, step, bind_index, bind, &bind.to,
                    ));
                };

                if target.step_id != step.id {
                    continue;
                }

                let Some(target_schema) = registered.contract.input_schema.get(&target.field)
                else {
                    return Err(invalid_reference_for_bind(
                        flow, step, bind_index, bind, &bind.to,
                    ));
                };
                let source_type = infer_source_type(
                    manifest, flow, registry, step_index, step, bind_index, bind, &bind.from,
                )?;

                if source_type != target_schema.field_type {
                    return Err(MocError::TypeMismatch {
                        flow_id: flow.id.clone(),
                        step_id: step.id.clone(),
                        bind_index,
                        from: bind.from.clone(),
                        to: bind.to.clone(),
                        expected: value_type_name(target_schema.field_type).to_string(),
                        actual: value_type_name(source_type).to_string(),
                    });
                }

                input_bindings.push(PlannedBinding {
                    flow_id: flow.id.clone(),
                    step_id: step.id.clone(),
                    bind_index,
                    from: bind.from.clone(),
                    to: bind.to.clone(),
                    to_field: target.field,
                });
            }

            for (field_name, schema) in &registered.contract.input_schema {
                if schema.required
                    && !input_bindings
                        .iter()
                        .any(|binding| binding.to_field == *field_name)
                {
                    return Err(MocError::MissingBind {
                        flow_id: flow.id.clone(),
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
        moc_input: &Value,
        step_outputs: &BTreeMap<String, Value>,
    ) -> Result<Map<String, Value>, MocError> {
        let mut step_input = Map::new();

        for binding in &self.input_bindings {
            let value = resolve_value(moc_input, step_outputs, binding)?;
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
    manifest: &MocManifest,
    flow: &Flow,
    registry: &Registry,
    target_step_index: usize,
    step: &FlowStep,
    bind_index: usize,
    bind: &Bind,
    reference: &str,
) -> Result<ValueType, MocError> {
    match parse_source_ref(reference)
        .ok_or_else(|| invalid_reference_for_bind(flow, step, bind_index, bind, reference))?
    {
        SourceRef::Input { field } => manifest
            .public_contract
            .input_schema
            .get(&field)
            .map(|schema| schema.field_type)
            .ok_or_else(|| invalid_reference_for_bind(flow, step, bind_index, bind, reference)),
        SourceRef::StepField { step_id, field } => {
            let source_index = flow
                .steps
                .iter()
                .position(|step| step.id == step_id)
                .ok_or_else(|| {
                    invalid_reference_for_bind(flow, step, bind_index, bind, reference)
                })?;

            if source_index >= target_step_index {
                return Err(invalid_reference_for_bind(
                    flow, step, bind_index, bind, reference,
                ));
            }

            let source_step = &flow.steps[source_index];
            let source_contract = &registry
                .get(&source_step.block)
                .ok_or_else(|| MocError::UnknownBlock(source_step.block.clone()))?
                .contract;

            source_contract
                .output_schema
                .get(&field)
                .map(|schema| schema.field_type)
                .ok_or_else(|| invalid_reference_for_bind(flow, step, bind_index, bind, reference))
        }
    }
}

fn resolve_value(
    input: &Value,
    step_outputs: &BTreeMap<String, Value>,
    binding: &PlannedBinding,
) -> Result<Value, MocError> {
    let reference = binding.from.as_str();
    match parse_source_ref(reference)
        .ok_or_else(|| invalid_reference_for_planned_binding(binding, reference))?
    {
        SourceRef::Input { field } => input
            .as_object()
            .and_then(|object| object.get(&field))
            .cloned()
            .ok_or_else(|| invalid_reference_for_planned_binding(binding, reference)),
        SourceRef::StepField { step_id, field } => step_outputs
            .get(&step_id)
            .and_then(|value| value.as_object())
            .and_then(|object| object.get(&field))
            .cloned()
            .ok_or_else(|| invalid_reference_for_planned_binding(binding, reference)),
    }
}

fn invalid_reference_for_bind(
    flow: &Flow,
    step: &FlowStep,
    bind_index: usize,
    bind: &Bind,
    reference: &str,
) -> MocError {
    invalid_reference(
        &flow.id, &step.id, bind_index, &bind.from, &bind.to, reference,
    )
}

fn invalid_reference_for_planned_binding(binding: &PlannedBinding, reference: &str) -> MocError {
    invalid_reference(
        &binding.flow_id,
        &binding.step_id,
        binding.bind_index,
        &binding.from,
        &binding.to,
        reference,
    )
}

fn invalid_reference(
    flow_id: &str,
    step_id: &str,
    bind_index: usize,
    from: &str,
    to: &str,
    reference: &str,
) -> MocError {
    MocError::InvalidReference {
        flow_id: flow_id.to_string(),
        step_id: step_id.to_string(),
        bind_index,
        from: from.to_string(),
        to: to.to_string(),
        reference: reference.to_string(),
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

impl fmt::Display for MocType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            MocType::RustLib => "rust_lib",
            MocType::FrontendLib => "frontend_lib",
            MocType::FrontendApp => "frontend_app",
            MocType::BackendApp => "backend_app",
        };

        formatter.write_str(value)
    }
}

impl fmt::Display for BackendMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            BackendMode::Console => "console",
            BackendMode::Service => "service",
        };

        formatter.write_str(value)
    }
}
