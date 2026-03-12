use std::path::Path;

use blocks_moc::{MocComposer, MocError};
use blocks_registry::Registry;

use crate::diagnostics::{RuleResult, SpanRange, ValidateReport};
use crate::ir::BclMocIr;

#[derive(Debug, Clone)]
pub struct ValidatedBcl {
    pub moc_id: String,
}

pub fn validate(
    blocks_root: &str,
    source_path: &str,
    ir: BclMocIr,
) -> Result<ValidatedBcl, ValidateReport> {
    let source = source_path.to_string();
    let registry = Registry::load_from_root(blocks_root).map_err(|error| {
        ValidateReport::error(
            source.clone(),
            RuleResult {
                error_id: "bcl.sema.registry_load_failed".to_string(),
                rule_id: "BCL-SEMA-REGISTRY-001".to_string(),
                severity: "error".to_string(),
                message: error.to_string(),
                hint: Some(
                    "ensure the blocks root exists and referenced block contracts are valid"
                        .to_string(),
                ),
                span: file_span(&ir),
            },
        )
    })?;

    ir.manifest
        .validate()
        .map_err(|error| map_moc_error(&source, &ir, &error))?;
    validate_known_blocks(&ir, &registry, source_path)?;
    validate_dependency_protocols(&ir, source_path)
        .map_err(|error| map_moc_error(&source, &ir, &error))?;
    if ir.manifest.has_validation_flow() {
        let _plan = MocComposer::new()
            .plan(&ir.manifest, &registry)
            .map_err(|error| map_moc_error(&source, &ir, &error))?;
    }

    Ok(ValidatedBcl {
        moc_id: ir.manifest.id.clone(),
    })
}

fn validate_known_blocks(
    ir: &BclMocIr,
    registry: &Registry,
    source_path: &str,
) -> Result<(), ValidateReport> {
    for flow in &ir.manifest.verification.flows {
        for step in &flow.steps {
            if ir
                .manifest
                .uses
                .internal_blocks
                .iter()
                .any(|item| item == &step.block)
            {
                continue;
            }
            if registry.get(&step.block).is_none() {
                return Err(ValidateReport::error(
                    source_path,
                    RuleResult {
                        error_id: "bcl.semantic.unknown_block".to_string(),
                        rule_id: "BCL-SEMA-003".to_string(),
                        severity: "error".to_string(),
                        message: format!("unknown block: {}", step.block),
                        hint: Some("declare the block in `uses { block ... }` and ensure it exists under the blocks root".to_string()),
                        span: ir
                            .spans
                            .step_spans
                            .get(&(flow.id.clone(), step.id.clone()))
                            .cloned()
                            .unwrap_or_else(|| file_span(ir)),
                    },
                ));
            }
        }
    }

    Ok(())
}

fn validate_dependency_protocols(ir: &BclMocIr, source_path: &str) -> Result<(), MocError> {
    let Some(moc_root) = Path::new(source_path).parent() else {
        return Err(MocError::InvalidDescriptor(
            "invalid moc.bcl path".to_string(),
        ));
    };
    let mocs_root = moc_root.parent().unwrap_or(moc_root);
    ir.manifest.validate_dependencies(mocs_root)
}

fn map_moc_error(source_path: &str, ir: &BclMocIr, error: &MocError) -> ValidateReport {
    match error {
        MocError::InvalidReference {
            flow_id,
            bind_index,
            reference,
            ..
        } => ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.semantic.invalid_reference".to_string(),
                rule_id: "BCL-SEMA-002".to_string(),
                severity: "error".to_string(),
                message: format!("invalid reference: {reference}"),
                hint: Some("references must use `input.<field>` or `<previous-step>.<field>` where the field exists in the declared schema".to_string()),
                span: ir
                    .spans
                    .bind_spans
                    .get(&(flow_id.clone(), *bind_index))
                    .cloned()
                    .unwrap_or_else(|| file_span(ir)),
            },
        ),
        MocError::UnknownBlock(block_id) => ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.semantic.unknown_block".to_string(),
                rule_id: "BCL-SEMA-003".to_string(),
                severity: "error".to_string(),
                message: format!("unknown block: {block_id}"),
                hint: Some("declare the block in `uses { block ... }` and ensure it exists under the blocks root".to_string()),
                span: file_span(ir),
            },
        ),
        MocError::TypeMismatch {
            flow_id,
            bind_index,
            expected,
            actual,
            ..
        } => ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.semantic.bind_type_mismatch".to_string(),
                rule_id: "BCL-SEMA-004".to_string(),
                severity: "error".to_string(),
                message: format!("bind type mismatch: expected {expected}, got {actual}"),
                hint: Some("fix the bind so the source field type matches the target field type".to_string()),
                span: ir
                    .spans
                    .bind_spans
                    .get(&(flow_id.clone(), *bind_index))
                    .cloned()
                    .unwrap_or_else(|| file_span(ir)),
            },
        ),
        MocError::MissingBind {
            flow_id,
            step_id,
            field,
        } => ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.semantic.missing_bind".to_string(),
                rule_id: "BCL-SEMA-005".to_string(),
                severity: "error".to_string(),
                message: format!("missing bind for required field {step_id}.{field}"),
                hint: Some("add a bind for every required block input before validating the flow".to_string()),
                span: ir
                    .spans
                    .step_spans
                    .get(&(flow_id.clone(), step_id.clone()))
                    .cloned()
                    .unwrap_or_else(|| file_span(ir)),
            },
        ),
        MocError::InvalidDescriptor(message)
            if message.starts_with("uses.blocks must exactly match") =>
        {
            ValidateReport::error(
                source_path,
                RuleResult {
                    error_id: "bcl.semantic.uses_blocks_flow_mismatch".to_string(),
                    rule_id: "BCL-SEMA-001".to_string(),
                    severity: "error".to_string(),
                    message: message.clone(),
                    hint: Some("ensure every external verification flow step block is declared exactly once in `uses { block ... }`".to_string()),
                    span: ir.spans.uses_span.clone().unwrap_or_else(|| file_span(ir)),
                },
            )
        }
        MocError::InvalidDescriptor(message)
            if is_dependency_error(message) || is_protocol_error(message) =>
        {
            let rule_id = if is_protocol_error(message) {
                "BCL-PROTO-001"
            } else {
                "BCL-PROTO-002"
            };
            ValidateReport::error(
                source_path,
                RuleResult {
                    error_id: "bcl.protocol.validation_failed".to_string(),
                    rule_id: rule_id.to_string(),
                    severity: "error".to_string(),
                    message: message.clone(),
                    hint: Some(
                        "ensure local and dependent moc protocol declarations match exactly"
                            .to_string(),
                    ),
                    span: dependency_span_for_message(ir, message).unwrap_or_else(|| file_span(ir)),
                },
            )
        }
        MocError::InvalidDescriptor(message) => ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.semantic.invalid_descriptor".to_string(),
                rule_id: "BCL-SEMA-000".to_string(),
                severity: "error".to_string(),
                message: message.clone(),
                hint: Some("fix the BCL descriptor so it lowers to a valid moc manifest".to_string()),
                span: file_span(ir),
            },
        ),
        other => ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.semantic.validation_failed".to_string(),
                rule_id: "BCL-SEMA-999".to_string(),
                severity: "error".to_string(),
                message: other.to_string(),
                hint: None,
                span: file_span(ir),
            },
        ),
    }
}

fn dependency_span_for_message(ir: &BclMocIr, message: &str) -> Option<SpanRange> {
    for dependency in &ir.spans.dependency_spans {
        if message.contains(&dependency.moc) && message.contains(&dependency.protocol) {
            return Some(dependency.span.clone());
        }
        if message.contains(&dependency.moc) {
            return Some(dependency.span.clone());
        }
    }
    None
}

fn is_dependency_error(message: &str) -> bool {
    message.starts_with("missing dependent moc manifest:")
        || message.starts_with("failed to read dependent moc manifest ")
        || message.starts_with("failed to parse moc manifest:")
        || message.starts_with("dependent moc ")
}

fn is_protocol_error(message: &str) -> bool {
    message.starts_with("protocol mismatch for dependency ")
        || message.starts_with("local protocol not declared for dependency ")
}

fn file_span(ir: &BclMocIr) -> SpanRange {
    ir.spans
        .file_span
        .clone()
        .unwrap_or_else(|| SpanRange::new(1, 1, 1, 1))
}
