mod diagnostics;
mod emit;
mod ir;
mod sema;
mod syntax;

use std::fs;

use blocks_moc::MocComposer;
use blocks_registry::Registry;

pub use diagnostics::{RuleResult, SpanRange, ValidateReport};
use ir::BclMocIr;
pub use ir::PlanReport;
pub use sema::ValidatedBcl;

#[derive(Debug, Clone)]
pub struct EmitResult {
    pub yaml: String,
}

pub fn validate_file(blocks_root: &str, source_path: &str) -> Result<ValidatedBcl, ValidateReport> {
    let source = fs::read_to_string(source_path).map_err(|error| {
        ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.io.read_failed".to_string(),
                rule_id: "BCL-IO-001".to_string(),
                severity: "error".to_string(),
                message: format!("failed to read BCL source {source_path}: {error}"),
                hint: None,
                span: SpanRange::new(1, 1, 1, 1),
            },
        )
    })?;
    validate_str(blocks_root, source_path, &source)
}

pub fn validate_str(
    blocks_root: &str,
    source_path: &str,
    source: &str,
) -> Result<ValidatedBcl, ValidateReport> {
    let ir = load_ir(source_path, source)?;
    sema::validate(blocks_root, source_path, ir)
}

pub fn success_report(source_path: &str) -> ValidateReport {
    ValidateReport::ok(source_path)
}

pub fn plan_file(blocks_root: &str, source_path: &str) -> Result<PlanReport, ValidateReport> {
    let source = read_source_file(source_path)?;
    plan_str(blocks_root, source_path, &source)
}

pub fn plan_str(
    blocks_root: &str,
    source_path: &str,
    source: &str,
) -> Result<PlanReport, ValidateReport> {
    let ir = load_validated_ir(blocks_root, source_path, source)?;
    let registry = Registry::load_from_root(blocks_root).map_err(|error| {
        ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.sema.registry_load_failed".to_string(),
                rule_id: "BCL-SEMA-REGISTRY-001".to_string(),
                severity: "error".to_string(),
                message: error.to_string(),
                hint: Some(
                    "ensure the blocks root exists and referenced block contracts are valid"
                        .to_string(),
                ),
                span: SpanRange::new(1, 1, 1, 1),
            },
        )
    })?;

    let execution_plan = if ir.manifest.has_validation_flow() {
        Some(
            MocComposer::new()
                .plan(&ir.manifest, &registry)
                .map_err(|error| {
                    ValidateReport::error(
                        source_path,
                        RuleResult {
                            error_id: "bcl.plan.build_failed".to_string(),
                            rule_id: "BCL-PLAN-001".to_string(),
                            severity: "error".to_string(),
                            message: error.to_string(),
                            hint: Some(
                                "ensure the BCL verification flow remains valid before planning"
                                    .to_string(),
                            ),
                            span: SpanRange::new(1, 1, 1, 1),
                        },
                    )
                })?,
        )
    } else {
        None
    };

    Ok(ir::build_plan_report(
        source_path,
        &ir.manifest,
        execution_plan.as_ref(),
    ))
}

pub fn emit_file(blocks_root: &str, source_path: &str) -> Result<EmitResult, ValidateReport> {
    let source = read_source_file(source_path)?;
    emit_str(blocks_root, source_path, &source)
}

pub fn emit_str(
    blocks_root: &str,
    source_path: &str,
    source: &str,
) -> Result<EmitResult, ValidateReport> {
    let ir = load_validated_ir(blocks_root, source_path, source)?;
    let yaml = emit::canonical_yaml(&ir.manifest).map_err(|message| {
        ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.emit.serialize_failed".to_string(),
                rule_id: "BCL-EMIT-001".to_string(),
                severity: "error".to_string(),
                message,
                hint: Some("ensure the lowered moc manifest remains serializable".to_string()),
                span: SpanRange::new(1, 1, 1, 1),
            },
        )
    })?;

    Ok(EmitResult { yaml })
}

pub fn check_against_file(emitted_yaml: &str, against_path: &str) -> Result<(), String> {
    let against_source = fs::read_to_string(against_path).map_err(|error| {
        format!("failed to read check-against manifest {against_path}: {error}")
    })?;
    emit::check_parity(emitted_yaml, &against_source)
}

fn read_source_file(source_path: &str) -> Result<String, ValidateReport> {
    fs::read_to_string(source_path).map_err(|error| {
        ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.io.read_failed".to_string(),
                rule_id: "BCL-IO-001".to_string(),
                severity: "error".to_string(),
                message: format!("failed to read BCL source {source_path}: {error}"),
                hint: None,
                span: SpanRange::new(1, 1, 1, 1),
            },
        )
    })
}

fn load_validated_ir(
    blocks_root: &str,
    source_path: &str,
    source: &str,
) -> Result<BclMocIr, ValidateReport> {
    let ir = load_ir(source_path, source)?;
    sema::validate(blocks_root, source_path, ir.clone())?;
    Ok(ir)
}

fn load_ir(source_path: &str, source: &str) -> Result<BclMocIr, ValidateReport> {
    let document = syntax::parser::parse(source_path, source)?;
    ir::lower(document).map_err(|message| {
        ValidateReport::error(
            source_path,
            RuleResult {
                error_id: "bcl.semantic.lower_failed".to_string(),
                rule_id: "BCL-SEMA-LOWER-001".to_string(),
                severity: "error".to_string(),
                message,
                hint: Some(
                    "ensure required BCL top-level statements are present and valid".to_string(),
                ),
                span: SpanRange::new(1, 1, 1, 1),
            },
        )
    })
}
