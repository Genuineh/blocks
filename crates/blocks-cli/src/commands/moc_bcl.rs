use std::fs;

use blocks_bcl::{check_against_file, emit_file, plan_file, success_report, validate_file};

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

fn render_validate_error(report: blocks_bcl::ValidateReport, json: bool) -> Result<String, String> {
    if json {
        let payload = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render bcl validation JSON: {error}"))?;
        Err(payload)
    } else {
        Err(render_report_human(report))
    }
}

fn render_report_human(report: blocks_bcl::ValidateReport) -> String {
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
