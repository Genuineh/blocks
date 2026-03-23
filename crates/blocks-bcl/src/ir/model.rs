use std::collections::BTreeMap;

use blocks_contract::{FieldSchema, ValueType};
use blocks_moc::{
    BackendMode, Bind, Flow, FlowStep, MocContract, MocDependency, MocManifest, MocProtocol,
    MocType, MocUses, MocVerification,
};

use crate::diagnostics::SpanRange;
use crate::syntax::ast::{BclDocument, FlowDecl, GuardClause, ProtocolDecl, RecoverClause, SchemaFieldDecl};

#[derive(Debug, Clone)]
pub struct BclMocIr {
    pub manifest: MocManifest,
    pub spans: SpanIndex,
}

#[derive(Debug, Clone, Default)]
pub struct SpanIndex {
    pub file_span: Option<SpanRange>,
    pub uses_span: Option<SpanRange>,
    pub dependency_spans: Vec<DependencySpan>,
    pub flow_spans: BTreeMap<String, SpanRange>,
    pub step_spans: BTreeMap<(String, String), SpanRange>,
    pub bind_spans: BTreeMap<(String, usize), SpanRange>,
    pub guard_clauses: BTreeMap<(String, String), GuardSpan>,
    pub recover_clauses: BTreeMap<String, RecoverSpan>,
}

#[derive(Debug, Clone)]
pub struct GuardSpan {
    pub condition: String,
    pub span: SpanRange,
}

#[derive(Debug, Clone)]
pub struct RecoverSpan {
    pub span: SpanRange,
}

#[derive(Debug, Clone)]
pub struct DependencySpan {
    pub moc: String,
    pub protocol: String,
    pub span: SpanRange,
}

pub fn lower(document: BclDocument) -> Result<BclMocIr, String> {
    let moc_type = parse_moc_type(
        document
            .type_spec
            .as_ref()
            .map(|item| item.moc_type.as_str()),
    )?;
    let backend_mode = parse_backend_mode(
        document
            .type_spec
            .as_ref()
            .and_then(|item| item.backend_mode.as_deref()),
    )?;
    let language = document
        .language
        .as_ref()
        .map(|item| item.value.clone())
        .ok_or_else(|| "missing required `language` statement".to_string())?;
    let entry = document
        .entry
        .as_ref()
        .map(|item| item.value.clone())
        .ok_or_else(|| "missing required `entry` statement".to_string())?;
    let name = document
        .name
        .as_ref()
        .map(|item| item.value.clone())
        .ok_or_else(|| "missing required `name` statement".to_string())?;

    let mut span_index = SpanIndex {
        file_span: Some(document.file_span.clone()),
        uses_span: document.uses.span.clone(),
        dependency_spans: document
            .dependencies
            .iter()
            .map(|dependency| DependencySpan {
                moc: dependency.moc.clone(),
                protocol: dependency.protocol.clone(),
                span: dependency.span.clone(),
            })
            .collect(),
        flow_spans: BTreeMap::new(),
        step_spans: BTreeMap::new(),
        bind_spans: BTreeMap::new(),
        guard_clauses: BTreeMap::new(),
        recover_clauses: BTreeMap::new(),
    };

    let mut entry_flow = None;
    let flows = document
        .verification
        .flows
        .iter()
        .map(|flow| lower_flow(flow, &mut span_index, &mut entry_flow))
        .collect::<Vec<_>>();
    let protocols = document
        .protocols
        .iter()
        .map(lower_protocol)
        .collect::<Result<Vec<_>, _>>()?;

    let manifest = MocManifest {
        id: document.moc_id,
        name,
        moc_type,
        backend_mode,
        language,
        entry,
        public_contract: MocContract {
            input_schema: lower_schema(&document.input_schema)?,
            output_schema: lower_schema(&document.output_schema)?,
        },
        uses: MocUses {
            blocks: document
                .uses
                .blocks
                .into_iter()
                .map(|item| item.value)
                .collect(),
            internal_blocks: document
                .uses
                .internal_blocks
                .into_iter()
                .map(|item| item.value)
                .collect(),
        },
        depends_on_mocs: document
            .dependencies
            .into_iter()
            .map(|dependency| MocDependency {
                moc: dependency.moc,
                protocol: dependency.protocol,
            })
            .collect(),
        protocols,
        verification: MocVerification {
            commands: document
                .verification
                .commands
                .into_iter()
                .map(|item| item.value)
                .collect(),
            entry_flow,
            flows,
        },
        acceptance_criteria: document
            .acceptance
            .into_iter()
            .map(|item| item.value)
            .collect(),
    };

    Ok(BclMocIr {
        manifest,
        spans: span_index,
    })
}

fn lower_flow(
    flow: &FlowDecl,
    span_index: &mut SpanIndex,
    entry_flow: &mut Option<String>,
) -> Flow {
    span_index
        .flow_spans
        .insert(flow.id.clone(), flow.span.clone());
    if flow.is_entry {
        *entry_flow = Some(flow.id.clone());
    }
    for step in &flow.steps {
        span_index
            .step_spans
            .insert((flow.id.clone(), step.id.clone()), step.span.clone());
        if let Some(ref guard) = step.guard {
            span_index.guard_clauses.insert(
                (flow.id.clone(), step.id.clone()),
                GuardSpan {
                    condition: guard.condition.clone(),
                    span: guard.span.clone(),
                },
            );
        }
    }
    for (index, bind) in flow.binds.iter().enumerate() {
        span_index
            .bind_spans
            .insert((flow.id.clone(), index + 1), bind.span.clone());
    }
    if let Some(ref recover) = flow.recover {
        span_index.recover_clauses.insert(
            flow.id.clone(),
            RecoverSpan {
                span: recover.span.clone(),
            },
        );
    }

    Flow {
        id: flow.id.clone(),
        steps: flow
            .steps
            .iter()
            .map(|step| FlowStep {
                id: step.id.clone(),
                block: step.block.clone(),
            })
            .collect(),
        binds: flow
            .binds
            .iter()
            .map(|bind| Bind {
                from: bind.from.clone(),
                to: bind.to.clone(),
            })
            .collect(),
    }
}

fn lower_protocol(protocol: &ProtocolDecl) -> Result<MocProtocol, String> {
    Ok(MocProtocol {
        name: protocol.name.clone(),
        channel: protocol.channel.value.clone(),
        input_schema: lower_schema(&protocol.input_fields)?,
        output_schema: lower_schema(&protocol.output_fields)?,
    })
}

fn lower_schema(fields: &[SchemaFieldDecl]) -> Result<BTreeMap<String, FieldSchema>, String> {
    let mut schema = BTreeMap::new();
    for field in fields {
        schema.insert(
            field.name.clone(),
            FieldSchema {
                field_type: parse_value_type(field.field_type.as_str())?,
                required: field.required,
                min_length: None,
                max_length: None,
                allowed_values: Vec::new(),
            },
        );
    }
    Ok(schema)
}

fn parse_moc_type(raw: Option<&str>) -> Result<MocType, String> {
    match raw {
        Some("rust_lib") => Ok(MocType::RustLib),
        Some("frontend_lib") => Ok(MocType::FrontendLib),
        Some("frontend_app") => Ok(MocType::FrontendApp),
        Some("backend_app") => Ok(MocType::BackendApp),
        Some(other) => Err(format!("unsupported moc type: {other}")),
        None => Err("missing required `type` statement".to_string()),
    }
}

fn parse_backend_mode(raw: Option<&str>) -> Result<Option<BackendMode>, String> {
    match raw {
        Some("console") => Ok(Some(BackendMode::Console)),
        Some("service") => Ok(Some(BackendMode::Service)),
        Some(other) => Err(format!("unsupported backend mode: {other}")),
        None => Ok(None),
    }
}

fn parse_value_type(raw: &str) -> Result<ValueType, String> {
    match raw {
        "string" => Ok(ValueType::String),
        "number" => Ok(ValueType::Number),
        "integer" => Ok(ValueType::Integer),
        "boolean" => Ok(ValueType::Boolean),
        "object" => Ok(ValueType::Object),
        "array" => Ok(ValueType::Array),
        other => Err(format!("unsupported field type: {other}")),
    }
}
