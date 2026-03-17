use std::fs;
use std::path::PathBuf;

use blocks_bcl::{
    PlanReport, ValidateReport, canonical_bcl, check_against_file, emit_file, format_file,
    plan_file, success_report, validate_file,
};
use blocks_moc::MocManifest;
use serde::Serialize;

use crate::app::toolchain::{read_text_file, resolve_descriptor_path, write_text_file};

#[derive(Default)]
struct BclValidateOptions {
    json: bool,
}

#[derive(Default)]
struct BclPlanOptions {
    json: bool,
}

#[derive(Default)]
struct BclEmitOptions {
    out: Option<String>,
    check_against: Option<String>,
}

#[derive(Default)]
struct BclCheckOptions {
    json: bool,
}

#[derive(Default)]
struct BclGraphOptions {
    json: bool,
}

#[derive(Default)]
struct BclExplainOptions {
    json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct BclGraphNode {
    id: String,
    kind: String,
    label: String,
}

#[derive(Debug, Clone, Serialize)]
struct BclGraphEdge {
    from: String,
    to: String,
    kind: String,
    label: String,
}

#[derive(Debug, Clone, Serialize)]
struct BclGraphReport {
    status: String,
    source: String,
    moc_id: String,
    descriptor_only: bool,
    nodes: Vec<BclGraphNode>,
    edges: Vec<BclGraphEdge>,
}

#[derive(Debug, Clone, Serialize)]
struct BclExplainIssue {
    error_id: String,
    rule_id: String,
    message: String,
    hint: Option<String>,
    span: blocks_bcl::SpanRange,
}

#[derive(Debug, Clone, Serialize)]
struct BclExplainReport {
    status: String,
    phase: String,
    source: String,
    moc_id: Option<String>,
    summary: String,
    issues: Vec<BclExplainIssue>,
    next_actions: Vec<String>,
}

pub fn init_command(path_arg: &str, args: &[String]) -> Result<String, String> {
    if !args.is_empty() {
        return Err(format!("unknown option for moc bcl init: {}", args[0]));
    }

    let manifest_path = resolve_moc_manifest_path(path_arg);
    let source = read_text_file(&manifest_path, "moc manifest")?;
    let manifest = MocManifest::from_yaml_str(&source).map_err(|error| {
        format!(
            "failed to load moc manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    let bcl_path = manifest_path
        .parent()
        .ok_or_else(|| format!("invalid moc manifest path: {}", manifest_path.display()))?
        .join("moc.bcl");
    if bcl_path.exists() {
        return Err(format!(
            "target path already exists: {}",
            bcl_path.display()
        ));
    }

    let rendered = canonical_bcl(&manifest).map_err(|error| {
        format!(
            "failed to scaffold BCL from {}: {error}",
            manifest_path.display()
        )
    })?;
    write_text_file(&bcl_path, &rendered)?;

    Ok(format!(
        "scaffolded bcl: {}\nsource_manifest: {}",
        bcl_path.display(),
        manifest_path.display()
    ))
}

pub fn fmt_command(source_path: &str) -> Result<String, String> {
    let source_path = resolve_bcl_source_path(source_path);
    let rendered = format_file(&source_path.display().to_string()).map_err(render_report_human)?;
    write_text_file(&source_path, &rendered)?;
    Ok(format!("formatted bcl source: {}", source_path.display()))
}

pub fn check_command(root: &str, source_path: &str, args: &[String]) -> Result<String, String> {
    let options = parse_check_options(args)?;
    let source_path = resolve_bcl_source_path(source_path);
    let source_path_string = source_path.display().to_string();

    match validate_file(root, &source_path_string) {
        Ok(validated) => {
            if options.json {
                let report = success_report(&source_path_string);
                serde_json::to_string_pretty(&report)
                    .map_err(|error| format!("failed to render bcl check JSON: {error}"))
            } else {
                Ok(format!(
                    "bcl check: ok\nmoc_id: {}\nsource: {}",
                    validated.moc_id,
                    source_path.display()
                ))
            }
        }
        Err(report) => render_validate_error(report, options.json),
    }
}

pub fn validate_command(root: &str, source_path: &str, args: &[String]) -> Result<String, String> {
    let options = parse_validate_options(args)?;
    match validate_file(root, source_path) {
        Ok(validated) => {
            if options.json {
                let report = success_report(source_path);
                serde_json::to_string_pretty(&report)
                    .map_err(|error| format!("failed to render bcl validation JSON: {error}"))
            } else {
                Ok(format!("valid bcl: {} ({source_path})", validated.moc_id))
            }
        }
        Err(report) => render_validate_error(report, options.json),
    }
}

pub fn plan_command(root: &str, source_path: &str, args: &[String]) -> Result<String, String> {
    let options = parse_plan_options(args)?;
    match plan_file(root, source_path) {
        Ok(report) => {
            if options.json {
                serde_json::to_string_pretty(&report)
                    .map_err(|error| format!("failed to render bcl plan JSON: {error}"))
            } else {
                let mut lines = vec![
                    format!("plan: {} ({source_path})", report.moc_id),
                    format!("type: {}", report.moc_type),
                    format!("descriptor_only: {}", report.descriptor_only),
                ];
                if let Some(flow) = &report.verification.plan {
                    lines.push(format!("flow: {}", flow.flow_id));
                    lines.push(format!("steps: {}", flow.steps.len()));
                } else {
                    lines.push("flow: none".to_string());
                }
                Ok(lines.join("\n"))
            }
        }
        Err(report) => render_validate_error(report, options.json),
    }
}

pub fn emit_command(root: &str, source_path: &str, args: &[String]) -> Result<String, String> {
    let options = parse_emit_options(args)?;
    let emitted = emit_file(root, source_path).map_err(render_report_human)?;

    if let Some(path) = &options.out {
        fs::write(path, &emitted.yaml)
            .map_err(|error| format!("failed to write emitted moc yaml {path}: {error}"))?;
    }

    if let Some(against_path) = &options.check_against {
        check_against_file(&emitted.yaml, against_path)
            .map_err(|error| format!("bcl parity failed against {against_path}: {error}"))?;
    }

    if let Some(path) = &options.out {
        let mut lines = vec![format!("emitted moc yaml: {path}")];
        if let Some(against_path) = &options.check_against {
            lines.push(format!("parity: matched {against_path}"));
        }
        return Ok(lines.join("\n"));
    }

    if let Some(against_path) = &options.check_against {
        return Ok(format!(
            "{}\nparity: matched {against_path}",
            emitted.yaml.trim_end()
        ));
    }

    Ok(emitted.yaml)
}

pub fn graph_command(root: &str, source_path: &str, args: &[String]) -> Result<String, String> {
    let options = parse_graph_options(args)?;
    let source_path = resolve_bcl_source_path(source_path);
    let source_path_string = source_path.display().to_string();
    match plan_file(root, &source_path_string) {
        Ok(report) => render_graph_report(build_graph_report(&report), options.json),
        Err(report) => render_validate_error(report, options.json),
    }
}

pub fn explain_command(root: &str, source_path: &str, args: &[String]) -> Result<String, String> {
    let options = parse_explain_options(args)?;
    let source_path = resolve_bcl_source_path(source_path);
    let source_path_string = source_path.display().to_string();
    match validate_file(root, &source_path_string) {
        Ok(validated) => match plan_file(root, &source_path_string) {
            Ok(plan) => render_explain_report(
                BclExplainReport {
                    status: "ok".to_string(),
                    phase: "plan".to_string(),
                    source: source_path_string,
                    moc_id: Some(validated.moc_id),
                    summary: explain_success_summary(&plan),
                    issues: Vec::new(),
                    next_actions: success_next_actions(&plan),
                },
                options.json,
            ),
            Err(report) => {
                render_explain_report(build_error_explain_report("plan", report), options.json)
            }
        },
        Err(report) => {
            render_explain_report(build_error_explain_report("validate", report), options.json)
        }
    }
}

fn render_validate_error(report: ValidateReport, json: bool) -> Result<String, String> {
    if json {
        let payload = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render bcl validation JSON: {error}"))?;
        Err(payload)
    } else {
        Err(render_report_human(report))
    }
}

fn resolve_moc_manifest_path(path_arg: &str) -> PathBuf {
    resolve_descriptor_path(path_arg, "moc.yaml")
}

fn resolve_bcl_source_path(path_arg: &str) -> PathBuf {
    resolve_descriptor_path(path_arg, "moc.bcl")
}

pub(crate) fn render_report_human(report: blocks_bcl::ValidateReport) -> String {
    let Some(first) = report.rule_results.first() else {
        return "bcl command failed without diagnostics".to_string();
    };
    let mut lines = vec![
        format!("bcl command failed: {}", first.message),
        format!("rule_id: {}", first.rule_id),
        format!("error_id: {}", first.error_id),
        format!(
            "span: {}:{}-{}:{}",
            first.span.line, first.span.column, first.span.end_line, first.span.end_column
        ),
    ];
    if let Some(hint) = &first.hint {
        lines.push(format!("hint: {hint}"));
    }
    lines.join("\n")
}

fn parse_validate_options(args: &[String]) -> Result<BclValidateOptions, String> {
    let mut options = BclValidateOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for moc bcl validate: {other}")),
        }
    }
    Ok(options)
}

fn parse_check_options(args: &[String]) -> Result<BclCheckOptions, String> {
    let mut options = BclCheckOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for moc bcl check: {other}")),
        }
    }
    Ok(options)
}

fn parse_plan_options(args: &[String]) -> Result<BclPlanOptions, String> {
    let mut options = BclPlanOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for moc bcl plan: {other}")),
        }
    }
    Ok(options)
}

fn parse_emit_options(args: &[String]) -> Result<BclEmitOptions, String> {
    let mut options = BclEmitOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--out" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--out requires a value".to_string())?;
                options.out = Some(value.clone());
                index += 2;
            }
            "--check-against" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--check-against requires a value".to_string())?;
                options.check_against = Some(value.clone());
                index += 2;
            }
            other => return Err(format!("unknown option for moc bcl emit: {other}")),
        }
    }
    Ok(options)
}

fn parse_graph_options(args: &[String]) -> Result<BclGraphOptions, String> {
    let mut options = BclGraphOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for moc bcl graph: {other}")),
        }
    }
    Ok(options)
}

fn parse_explain_options(args: &[String]) -> Result<BclExplainOptions, String> {
    let mut options = BclExplainOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for moc bcl explain: {other}")),
        }
    }
    Ok(options)
}

fn build_graph_report(report: &PlanReport) -> BclGraphReport {
    let mut nodes = vec![BclGraphNode {
        id: format!("moc:{}", report.moc_id),
        kind: "moc".to_string(),
        label: report.moc_id.clone(),
    }];
    let mut edges = Vec::new();

    for block_id in &report.uses.blocks {
        nodes.push(BclGraphNode {
            id: format!("block:{block_id}"),
            kind: "block".to_string(),
            label: block_id.clone(),
        });
        edges.push(BclGraphEdge {
            from: format!("moc:{}", report.moc_id),
            to: format!("block:{block_id}"),
            kind: "uses".to_string(),
            label: "uses".to_string(),
        });
    }
    for dependency in &report.dependencies {
        nodes.push(BclGraphNode {
            id: format!("dependency:{}", dependency.moc),
            kind: "moc_dependency".to_string(),
            label: dependency.moc.clone(),
        });
        edges.push(BclGraphEdge {
            from: format!("moc:{}", report.moc_id),
            to: format!("dependency:{}", dependency.moc),
            kind: "depends_on".to_string(),
            label: dependency.protocol.clone(),
        });
    }
    for protocol in &report.protocols {
        nodes.push(BclGraphNode {
            id: format!("protocol:{}", protocol.name),
            kind: "protocol".to_string(),
            label: protocol.name.clone(),
        });
        edges.push(BclGraphEdge {
            from: format!("moc:{}", report.moc_id),
            to: format!("protocol:{}", protocol.name),
            kind: "exposes".to_string(),
            label: protocol.channel.clone(),
        });
    }
    if let Some(plan) = &report.verification.plan {
        let flow_id = format!("flow:{}", plan.flow_id);
        nodes.push(BclGraphNode {
            id: flow_id.clone(),
            kind: "flow".to_string(),
            label: plan.flow_id.clone(),
        });
        edges.push(BclGraphEdge {
            from: format!("moc:{}", report.moc_id),
            to: flow_id.clone(),
            kind: "verification_flow".to_string(),
            label: report
                .verification
                .entry_flow
                .clone()
                .unwrap_or_else(|| "flow".to_string()),
        });
        for step in &plan.steps {
            let step_id = format!("step:{}:{}", plan.flow_id, step.id);
            nodes.push(BclGraphNode {
                id: step_id.clone(),
                kind: "step".to_string(),
                label: format!("{} ({})", step.id, step.block),
            });
            edges.push(BclGraphEdge {
                from: flow_id.clone(),
                to: step_id.clone(),
                kind: "contains".to_string(),
                label: step.block.clone(),
            });
            for binding in &step.input_bindings {
                edges.push(BclGraphEdge {
                    from: binding.from.clone(),
                    to: binding.to.clone(),
                    kind: "bind".to_string(),
                    label: binding.to_field.clone(),
                });
            }
        }
    }

    BclGraphReport {
        status: "ok".to_string(),
        source: report.source.clone(),
        moc_id: report.moc_id.clone(),
        descriptor_only: report.descriptor_only,
        nodes,
        edges,
    }
}

fn render_graph_report(report: BclGraphReport, json_output: bool) -> Result<String, String> {
    if json_output {
        return serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render bcl graph JSON: {error}"));
    }

    let mut lines = vec![
        format!("bcl graph: {}", report.moc_id),
        format!("descriptor_only: {}", report.descriptor_only),
        format!("nodes: {}", report.nodes.len()),
        format!("edges: {}", report.edges.len()),
    ];
    for edge in &report.edges {
        lines.push(format!(
            "edge {} -> {} [{}:{}]",
            edge.from, edge.to, edge.kind, edge.label
        ));
    }
    Ok(lines.join("\n"))
}

fn build_error_explain_report(phase: &str, report: ValidateReport) -> BclExplainReport {
    let issues = report
        .rule_results
        .iter()
        .map(|result| BclExplainIssue {
            error_id: result.error_id.clone(),
            rule_id: result.rule_id.clone(),
            message: result.message.clone(),
            hint: result.hint.clone(),
            span: result.span.clone(),
        })
        .collect::<Vec<_>>();
    let next_actions = issues
        .iter()
        .filter_map(|issue| issue.hint.clone())
        .collect::<Vec<_>>();
    BclExplainReport {
        status: "error".to_string(),
        phase: phase.to_string(),
        source: report.source,
        moc_id: None,
        summary: format!("BCL {} failed with {} issue(s)", phase, issues.len()),
        issues,
        next_actions,
    }
}

fn explain_success_summary(plan: &PlanReport) -> String {
    if let Some(flow) = &plan.verification.plan {
        format!(
            "BCL source is valid and lowers to `{}` with verification flow `{}` containing {} step(s)",
            plan.moc_id,
            flow.flow_id,
            flow.steps.len()
        )
    } else {
        format!(
            "BCL source is valid and lowers to descriptor-only moc `{}` with no verification flow",
            plan.moc_id
        )
    }
}

fn success_next_actions(plan: &PlanReport) -> Vec<String> {
    let mut actions =
        vec!["run `blocks moc bcl graph` to inspect the lowered assembly graph".to_string()];
    if plan.descriptor_only {
        actions
            .push("add verification.entry_flow only after the runtime path is clear".to_string());
    } else {
        actions.push("run `blocks conformance run bcl --check-against <moc.yaml>` before tightening gate mode".to_string());
    }
    actions
}

fn render_explain_report(report: BclExplainReport, json_output: bool) -> Result<String, String> {
    if json_output {
        let rendered = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render bcl explain JSON: {error}"))?;
        return if report.status == "error" {
            Err(rendered)
        } else {
            Ok(rendered)
        };
    }

    let mut lines = vec![
        format!("bcl explain: {}", report.status),
        format!("phase: {}", report.phase),
        format!("source: {}", report.source),
        format!("summary: {}", report.summary),
    ];
    if let Some(moc_id) = &report.moc_id {
        lines.push(format!("moc_id: {moc_id}"));
    }
    for issue in &report.issues {
        lines.push(format!(
            "issue: {} {} {}",
            issue.rule_id, issue.error_id, issue.message
        ));
        if let Some(hint) = &issue.hint {
            lines.push(format!("hint: {hint}"));
        }
    }
    for action in &report.next_actions {
        lines.push(format!("next: {action}"));
    }
    if report.status == "error" {
        Err(lines.join("\n"))
    } else {
        Ok(lines.join("\n"))
    }
}
