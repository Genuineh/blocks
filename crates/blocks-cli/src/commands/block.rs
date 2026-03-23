use std::path::{Path, PathBuf};

use blocks_contract::{BlockContract, ContractLoadError, ImplementationKind, ImplementationTarget};
use blocks_runtime::{DiagnosticEvent, read_diagnostic_artifact, read_diagnostic_events};
use serde::Serialize;
use serde_json::json;

use crate::app::resolve_diagnostics_root;
use crate::app::toolchain::{
    block_readme_template, block_rust_cargo_toml, block_rust_lib_template,
    block_tauri_entry_template, build_block_contract_template, count_files, ensure_directory,
    ensure_new_directory, normalize_yaml, parse_block_kind, parse_block_target, read_text_file,
    resolve_descriptor_path, resolve_workspace_root, run_shell_command, run_shell_script,
    validate_block_scaffold_shape, write_text_file,
};
use crate::render::render_block_diagnose_human;

#[derive(Default)]
struct BlockInitOptions {
    kind: Option<ImplementationKind>,
    target: Option<ImplementationTarget>,
}

#[derive(Default)]
struct BlockCheckOptions {
    json: bool,
}

#[derive(Default)]
struct BlockEvidenceOptions {
    json: bool,
}

#[derive(Default)]
struct BlockDoctorOptions {
    json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct BlockDoctorEvidence {
    tests_files: usize,
    examples_files: usize,
    evaluators_files: usize,
    fixtures_files: usize,
    tests_runner: bool,
    examples_runner: bool,
    evaluators_runner: bool,
}

#[derive(Debug, Clone, Serialize)]
struct BlockDoctorDiagnostic {
    execution_id: String,
    status: String,
    trace_id: Option<String>,
    error_id: Option<String>,
    duration_ms: Option<u128>,
    artifact_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BlockDoctorReport {
    target_kind: String,
    status: String,
    block_id: Option<String>,
    path: String,
    check_status: String,
    warnings: Vec<String>,
    errors: Vec<String>,
    evidence: BlockDoctorEvidence,
    latest_diagnostic: Option<BlockDoctorDiagnostic>,
    recommendations: Vec<String>,
}

pub fn init_command(blocks_root: &str, block_id: &str, args: &[String]) -> Result<String, String> {
    let options = parse_init_options(args)?;
    let kind = options.kind.unwrap_or(ImplementationKind::Rust);
    let target = options.target.unwrap_or(ImplementationTarget::Shared);
    validate_block_scaffold_shape(kind, target)?;

    let block_root = Path::new(blocks_root).join(block_id);
    ensure_new_directory(&block_root)?;
    ensure_directory(&block_root.join("tests"))?;
    ensure_directory(&block_root.join("examples"))?;
    ensure_directory(&block_root.join("evaluators"))?;
    ensure_directory(&block_root.join("fixtures"))?;

    let contract = build_block_contract_template(block_id, kind, target);
    let contract_yaml = normalize_yaml(&contract, "block contract")?;
    let contract_path = block_root.join("block.yaml");
    write_text_file(&contract_path, &contract_yaml)?;
    write_text_file(
        &block_root.join("README.md"),
        &block_readme_template(block_id, kind, target),
    )?;

    match kind {
        ImplementationKind::Rust => {
            let rust_root = block_root.join("rust");
            ensure_directory(&rust_root)?;
            write_text_file(&rust_root.join("lib.rs"), block_rust_lib_template())?;
            write_text_file(
                &rust_root.join("Cargo.toml"),
                &block_rust_cargo_toml(block_id),
            )?;
        }
        ImplementationKind::TauriTs => {
            let source_root = block_root.join("tauri_ts").join("src");
            ensure_directory(&source_root)?;
            write_text_file(&source_root.join("index.ts"), block_tauri_entry_template())?;
        }
    }

    Ok(format!(
        "scaffolded block: {}\ncontract: {}",
        block_root.display(),
        contract_path.display()
    ))
}

pub fn fmt_command(path_arg: &str) -> Result<String, String> {
    let contract_path = resolve_block_contract_path(path_arg);
    let source = read_text_file(&contract_path, "block contract")?;
    let (contract, _) = BlockContract::from_yaml_str_with_report(&source).map_err(|error| {
        format!(
            "failed to format block contract {}: {error}",
            contract_path.display()
        )
    })?;
    let rendered = normalize_yaml(&contract, "block contract")?;
    write_text_file(&contract_path, &rendered)?;
    Ok(format!(
        "formatted block contract: {}",
        contract_path.display()
    ))
}

pub fn check_command(path_arg: &str, args: &[String]) -> Result<String, String> {
    let options = parse_check_options(args)?;
    let contract_path = resolve_block_contract_path(path_arg);
    let source = read_text_file(&contract_path, "block contract")?;

    let (contract, report) = match BlockContract::from_yaml_str_with_report(&source) {
        Ok(result) => result,
        Err(ContractLoadError::Parse(error)) => {
            return render_check_failure(
                options.json,
                &contract_path,
                None,
                &[format!("failed to parse block contract: {error}")],
                &[],
            );
        }
        Err(ContractLoadError::InvalidDefinition(message)) => {
            return render_check_failure(options.json, &contract_path, None, &[message], &[]);
        }
    };

    let block_root = contract_path
        .parent()
        .ok_or_else(|| format!("invalid block contract path: {}", contract_path.display()))?;
    let mut errors = Vec::new();
    let warnings = report
        .warnings()
        .into_iter()
        .map(|issue| format!("{}: {}", issue.path, issue.message))
        .collect::<Vec<_>>();

    if let Some(implementation) = &contract.implementation {
        let implementation_path = block_root.join(&implementation.entry);
        if !implementation_path.is_file() {
            errors.push(format!(
                "missing implementation entry: {}",
                implementation_path.display()
            ));
        }
    }

    let status = if errors.is_empty() {
        if warnings.is_empty() { "ok" } else { "warn" }
    } else {
        "error"
    };

    if options.json {
        let payload = json!({
            "status": status,
            "kind": "block",
            "path": contract_path.display().to_string(),
            "block_id": contract.id,
            "implementation": contract.implementation.as_ref().map(|implementation| {
                json!({
                    "kind": match implementation.kind {
                        ImplementationKind::Rust => "rust",
                        ImplementationKind::TauriTs => "tauri_ts",
                    },
                    "target": match implementation.target {
                        ImplementationTarget::Backend => "backend",
                        ImplementationTarget::Frontend => "frontend",
                        ImplementationTarget::Shared => "shared",
                    },
                    "entry": implementation.entry,
                })
            }),
            "warnings": warnings,
            "errors": errors,
        });
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("failed to render block check JSON: {error}"))?;
        return if status == "error" {
            Err(rendered)
        } else {
            Ok(rendered)
        };
    }

    let mut lines = vec![
        format!("block check: {status}"),
        format!("id: {}", contract.id),
        format!("path: {}", contract_path.display()),
    ];
    if let Some(implementation) = &contract.implementation {
        lines.push(format!(
            "implementation: {:?} {:?} {}",
            implementation.kind, implementation.target, implementation.entry
        ));
    }
    for warning in &warnings {
        lines.push(format!("warning: {warning}"));
    }
    for error in &errors {
        lines.push(format!("error: {error}"));
    }
    let rendered = lines.join("\n");
    if status == "error" {
        Err(rendered)
    } else {
        Ok(rendered)
    }
}

pub fn test_command(path_arg: &str, args: &[String]) -> Result<String, String> {
    let options = parse_evidence_options(args, "block test")?;
    let contract_path = resolve_block_contract_path(path_arg);
    let (contract, contract_warnings) =
        load_block_contract_for_evidence(&contract_path, options.json)?;
    let block_root = contract_path
        .parent()
        .ok_or_else(|| format!("invalid block contract path: {}", contract_path.display()))?;
    let workspace_root = resolve_workspace_root(block_root);

    let tests_script = block_root.join("tests").join("run.sh");
    let examples_script = block_root.join("examples").join("run.sh");
    let automated_commands = verification_automated_commands(&contract);

    let mut cases = Vec::new();
    if tests_script.is_file() {
        cases.push(run_case_script(
            "tests_runner",
            "tests/run.sh",
            &tests_script,
            block_root,
        )?);
    }
    if examples_script.is_file() {
        cases.push(run_case_script(
            "examples_runner",
            "examples/run.sh",
            &examples_script,
            block_root,
        )?);
    }
    if cases.is_empty() {
        for command in automated_commands {
            cases.push(run_case_command(
                "verification_command",
                command.as_str(),
                command.as_str(),
                &workspace_root,
            )?);
        }
    }

    if cases.is_empty() {
        return render_evidence_failure(
            options.json,
            "test",
            &contract_path,
            Some(&contract.id),
            &[String::from(
                "no executable block test evidence configured; expected tests/run.sh, examples/run.sh, or verification.automated",
            )],
            &contract_warnings,
        );
    }

    render_evidence_report(
        options.json,
        "test",
        &contract,
        &contract_path,
        contract_warnings,
        cases,
    )
}

pub fn eval_command(path_arg: &str, args: &[String]) -> Result<String, String> {
    let options = parse_evidence_options(args, "block eval")?;
    let contract_path = resolve_block_contract_path(path_arg);
    let (contract, contract_warnings) =
        load_block_contract_for_evidence(&contract_path, options.json)?;
    let block_root = contract_path
        .parent()
        .ok_or_else(|| format!("invalid block contract path: {}", contract_path.display()))?;
    let workspace_root = resolve_workspace_root(block_root);

    let evaluator_script = block_root.join("evaluators").join("run.sh");
    let evaluation_commands = evaluation_commands(&contract);
    let mut cases = Vec::new();

    if evaluator_script.is_file() {
        cases.push(run_case_script(
            "evaluator_runner",
            "evaluators/run.sh",
            &evaluator_script,
            block_root,
        )?);
    }
    if cases.is_empty() {
        for command in evaluation_commands {
            cases.push(run_case_command(
                "evaluation_command",
                command.as_str(),
                command.as_str(),
                &workspace_root,
            )?);
        }
    }

    if cases.is_empty() {
        return render_evidence_failure(
            options.json,
            "eval",
            &contract_path,
            Some(&contract.id),
            &[String::from(
                "no executable block evaluation configured; expected evaluators/run.sh or evaluation.commands",
            )],
            &contract_warnings,
        );
    }

    render_evidence_report(
        options.json,
        "eval",
        &contract,
        &contract_path,
        contract_warnings,
        cases,
    )
}

pub fn doctor_command(blocks_root: &str, target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_doctor_options(args, "block doctor")?;
    let contract_path = resolve_block_doctor_path(blocks_root, target);
    let path_string = contract_path.display().to_string();
    let check_payload =
        decode_doctor_payload(check_command(&path_string, &[String::from("--json")]))?;
    let block_id = check_payload
        .get("block_id")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
        .or_else(|| (!Path::new(target).exists()).then_some(target.to_string()));
    let warnings = check_payload
        .get("warnings")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let errors = check_payload
        .get("errors")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let block_root = contract_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(blocks_root).join(target));
    let evidence = BlockDoctorEvidence {
        tests_files: count_files(&block_root.join("tests")),
        examples_files: count_files(&block_root.join("examples")),
        evaluators_files: count_files(&block_root.join("evaluators")),
        fixtures_files: count_files(&block_root.join("fixtures")),
        tests_runner: block_root.join("tests").join("run.sh").is_file(),
        examples_runner: block_root.join("examples").join("run.sh").is_file(),
        evaluators_runner: block_root.join("evaluators").join("run.sh").is_file(),
    };
    let latest_diagnostic = block_id
        .as_deref()
        .map(|id| latest_block_diagnostic(blocks_root, id))
        .transpose()?
        .flatten();
    let recommendations =
        build_block_doctor_recommendations(&errors, &warnings, &evidence, &latest_diagnostic);
    let status = if !errors.is_empty() {
        "error"
    } else if !warnings.is_empty() || !recommendations.is_empty() {
        "warn"
    } else {
        "ok"
    };
    let report = BlockDoctorReport {
        target_kind: "block".to_string(),
        status: status.to_string(),
        block_id,
        path: path_string,
        check_status: check_payload
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_string(),
        warnings,
        errors,
        evidence,
        latest_diagnostic,
        recommendations,
    };
    render_block_doctor_report(report, options.json)
}

#[derive(Default)]
struct BlockDiagnoseOptions {
    execution_id: Option<String>,
    json: bool,
}

pub fn diagnose_command(
    blocks_root: &str,
    block_id: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_diagnose_options(args)?;
    let diagnostics_root = resolve_diagnostics_root(blocks_root);
    let events = read_diagnostic_events(&diagnostics_root)?;
    let block_events: Vec<DiagnosticEvent> = events
        .into_iter()
        .filter(|event| event.block_id == block_id)
        .collect();
    if block_events.is_empty() {
        return Err(format!("no diagnostics found for block {block_id}"));
    }

    let selected_execution_id = if let Some(execution_id) = options.execution_id {
        execution_id
    } else {
        select_latest_execution_id(&block_events)
            .ok_or_else(|| format!("no diagnostic execution found for block {block_id}"))?
    };
    let selected_events: Vec<DiagnosticEvent> = block_events
        .into_iter()
        .filter(|event| event.execution_id == selected_execution_id)
        .collect();
    let artifact = read_diagnostic_artifact(&diagnostics_root, &selected_execution_id)?;

    if options.json {
        return serde_json::to_string_pretty(&json!({
            "block_id": block_id,
            "diagnostics_root": diagnostics_root,
            "execution_id": selected_execution_id,
            "events": selected_events,
            "artifact": artifact
        }))
        .map_err(|error| format!("failed to render diagnostic JSON: {error}"));
    }

    render_block_diagnose_human(
        block_id,
        &selected_execution_id,
        diagnostics_root.as_path(),
        &selected_events,
        artifact.as_ref(),
    )
}

fn parse_diagnose_options(args: &[String]) -> Result<BlockDiagnoseOptions, String> {
    let mut options = BlockDiagnoseOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--latest" => {
                options.execution_id = None;
                index += 1;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--execution-id" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--execution-id requires a value".to_string())?;
                options.execution_id = Some(value.clone());
                index += 2;
            }
            other => {
                return Err(format!("unknown option for block diagnose: {other}"));
            }
        }
    }
    Ok(options)
}

fn parse_doctor_options(args: &[String], label: &str) -> Result<BlockDoctorOptions, String> {
    let mut options = BlockDoctorOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for {label}: {other}")),
        }
    }
    Ok(options)
}

fn load_block_contract_for_evidence(
    contract_path: &Path,
    json_output: bool,
) -> Result<(BlockContract, Vec<String>), String> {
    let source = read_text_file(contract_path, "block contract")?;
    match BlockContract::from_yaml_str_with_report(&source) {
        Ok((contract, report)) => Ok((
            contract,
            report
                .warnings()
                .into_iter()
                .map(|issue| format!("{}: {}", issue.path, issue.message))
                .collect(),
        )),
        Err(ContractLoadError::Parse(error)) => match render_evidence_failure(
            json_output,
            "check",
            contract_path,
            None,
            &[format!("failed to parse block contract: {error}")],
            &[],
        ) {
            Ok(_) => unreachable!(),
            Err(message) => Err(message),
        },
        Err(ContractLoadError::InvalidDefinition(message)) => match render_evidence_failure(
            json_output,
            "check",
            contract_path,
            None,
            &[message],
            &[],
        ) {
            Ok(_) => unreachable!(),
            Err(message) => Err(message),
        },
    }
}

fn select_latest_execution_id(events: &[DiagnosticEvent]) -> Option<String> {
    events
        .iter()
        .max_by_key(|event| event.timestamp_ms)
        .map(|event| event.execution_id.clone())
}

fn resolve_block_contract_path(path_arg: &str) -> PathBuf {
    resolve_descriptor_path(path_arg, "block.yaml")
}

fn parse_init_options(args: &[String]) -> Result<BlockInitOptions, String> {
    let mut options = BlockInitOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--kind" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--kind requires a value".to_string())?;
                options.kind = Some(parse_block_kind(value)?);
                index += 2;
            }
            "--target" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--target requires a value".to_string())?;
                options.target = Some(parse_block_target(value)?);
                index += 2;
            }
            other => return Err(format!("unknown option for block init: {other}")),
        }
    }
    Ok(options)
}

fn parse_check_options(args: &[String]) -> Result<BlockCheckOptions, String> {
    let mut options = BlockCheckOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for block check: {other}")),
        }
    }
    Ok(options)
}

fn parse_evidence_options(args: &[String], label: &str) -> Result<BlockEvidenceOptions, String> {
    let mut options = BlockEvidenceOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for {label}: {other}")),
        }
    }
    Ok(options)
}

fn verification_automated_commands(contract: &BlockContract) -> Vec<String> {
    contract
        .verification
        .get("automated")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect()
}

fn evaluation_commands(contract: &BlockContract) -> Vec<String> {
    contract
        .evaluation
        .get("commands")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect()
}

fn run_case_script(
    kind: &str,
    label: &str,
    script_path: &Path,
    cwd: &Path,
) -> Result<serde_json::Value, String> {
    let result = run_shell_script(script_path, cwd)?;
    Ok(case_result(
        kind,
        label,
        result.exit_code,
        &result.stdout,
        &result.stderr,
    ))
}

fn run_case_command(
    kind: &str,
    label: &str,
    command: &str,
    cwd: &Path,
) -> Result<serde_json::Value, String> {
    let result = run_shell_command(command, cwd)?;
    Ok(case_result(
        kind,
        label,
        result.exit_code,
        &result.stdout,
        &result.stderr,
    ))
}

fn case_result(
    kind: &str,
    label: &str,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> serde_json::Value {
    let status = if exit_code == Some(0) { "ok" } else { "error" };
    json!({
        "kind": kind,
        "label": label,
        "status": status,
        "exit_code": exit_code,
        "stdout": stdout,
        "stderr": stderr,
    })
}

fn render_evidence_report(
    json_output: bool,
    suite: &str,
    contract: &BlockContract,
    contract_path: &Path,
    warnings: Vec<String>,
    cases: Vec<serde_json::Value>,
) -> Result<String, String> {
    let block_root = contract_path
        .parent()
        .ok_or_else(|| format!("invalid block contract path: {}", contract_path.display()))?;
    let evidence = json!({
        "tests_files": count_files(&block_root.join("tests")),
        "examples_files": count_files(&block_root.join("examples")),
        "evaluators_files": count_files(&block_root.join("evaluators")),
        "fixtures_files": count_files(&block_root.join("fixtures")),
    });
    let failures = cases
        .iter()
        .filter(|case| case["status"] == "error")
        .map(|case| {
            json!({
                "label": case["label"],
                "message": case["stderr"].as_str().filter(|value| !value.is_empty()).unwrap_or_else(|| case["stdout"].as_str().unwrap_or("command failed")),
                "exit_code": case["exit_code"],
            })
        })
        .collect::<Vec<_>>();
    let status = if failures.is_empty() { "ok" } else { "error" };

    let payload = json!({
        "status": status,
        "suite": suite,
        "block_id": contract.id,
        "path": contract_path.display().to_string(),
        "evidence": evidence,
        "warnings": warnings,
        "cases_run": cases.len(),
        "cases": cases,
        "failures": failures,
    });

    if json_output {
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("failed to render block {suite} JSON: {error}"))?;
        return if status == "error" {
            Err(rendered)
        } else {
            Ok(rendered)
        };
    }

    let mut lines = vec![
        format!("block {suite}: {status}"),
        format!("id: {}", contract.id),
        format!("path: {}", contract_path.display()),
        format!("cases_run: {}", payload["cases_run"]),
        format!(
            "evidence: tests={} examples={} evaluators={} fixtures={}",
            payload["evidence"]["tests_files"],
            payload["evidence"]["examples_files"],
            payload["evidence"]["evaluators_files"],
            payload["evidence"]["fixtures_files"]
        ),
    ];
    for warning in payload["warnings"].as_array().into_iter().flatten() {
        if let Some(warning) = warning.as_str() {
            lines.push(format!("warning: {warning}"));
        }
    }
    for failure in payload["failures"].as_array().into_iter().flatten() {
        let label = failure["label"].as_str().unwrap_or("unknown");
        let message = failure["message"].as_str().unwrap_or("command failed");
        lines.push(format!("error: {label}: {message}"));
    }

    let rendered = lines.join("\n");
    if status == "error" {
        Err(rendered)
    } else {
        Ok(rendered)
    }
}

fn render_evidence_failure(
    json_output: bool,
    suite: &str,
    contract_path: &Path,
    block_id: Option<&str>,
    errors: &[String],
    warnings: &[String],
) -> Result<String, String> {
    if json_output {
        let payload = json!({
            "status": "error",
            "suite": suite,
            "path": contract_path.display().to_string(),
            "block_id": block_id,
            "warnings": warnings,
            "cases_run": 0,
            "cases": [],
            "failures": errors.iter().map(|message| json!({ "label": suite, "message": message })).collect::<Vec<_>>(),
        });
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("failed to render block {suite} JSON: {error}"))?;
        Err(rendered)
    } else {
        let mut lines = vec![
            format!("block {suite}: error"),
            format!("path: {}", contract_path.display()),
        ];
        if let Some(block_id) = block_id {
            lines.push(format!("id: {block_id}"));
        }
        for warning in warnings {
            lines.push(format!("warning: {warning}"));
        }
        for error in errors {
            lines.push(format!("error: {error}"));
        }
        Err(lines.join("\n"))
    }
}

fn decode_doctor_payload(result: Result<String, String>) -> Result<serde_json::Value, String> {
    let payload = match result {
        Ok(payload) | Err(payload) => payload,
    };
    serde_json::from_str(&payload)
        .map_err(|error| format!("failed to decode block doctor JSON payload: {error}"))
}

fn resolve_block_doctor_path(blocks_root: &str, target: &str) -> PathBuf {
    if Path::new(target).exists() {
        resolve_block_contract_path(target)
    } else {
        Path::new(blocks_root).join(target).join("block.yaml")
    }
}

fn latest_block_diagnostic(
    blocks_root: &str,
    block_id: &str,
) -> Result<Option<BlockDoctorDiagnostic>, String> {
    let diagnostics_root = resolve_diagnostics_root(blocks_root);
    let events_path = diagnostics_root.join("events.jsonl");
    if !events_path.is_file() {
        return Ok(None);
    }

    let events = read_diagnostic_events(&diagnostics_root)?;
    let latest = events
        .into_iter()
        .filter(|event| event.block_id == block_id)
        .max_by_key(|event| event.timestamp_ms);
    let Some(event) = latest else {
        return Ok(None);
    };

    let artifact = read_diagnostic_artifact(&diagnostics_root, &event.execution_id)?;
    Ok(Some(BlockDoctorDiagnostic {
        execution_id: event.execution_id.clone(),
        status: if event.event == "block.execution.failure" {
            "failure".to_string()
        } else {
            "success".to_string()
        },
        trace_id: event.trace_id.clone(),
        error_id: event.error_id.clone(),
        duration_ms: event.duration_ms,
        artifact_path: artifact.map(|_| {
            diagnostics_root
                .join("artifacts")
                .join(format!("{}.json", event.execution_id))
                .display()
                .to_string()
        }),
    }))
}

fn build_block_doctor_recommendations(
    errors: &[String],
    warnings: &[String],
    evidence: &BlockDoctorEvidence,
    latest_diagnostic: &Option<BlockDoctorDiagnostic>,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    if !errors.is_empty() {
        recommendations
            .push("fix `blocks block check` errors before running conformance".to_string());
    }
    if evidence.tests_files == 0 || !evidence.tests_runner {
        recommendations.push(
            "add `tests/run.sh` and at least one checked-in test asset for deterministic block verification"
                .to_string(),
        );
    }
    if evidence.examples_files == 0 || !evidence.examples_runner {
        recommendations.push(
            "add `examples/run.sh` plus a minimal success example so AI and CI can execute the block deterministically"
                .to_string(),
        );
    }
    if evidence.evaluators_files == 0 || !evidence.evaluators_runner {
        recommendations.push(
            "add `evaluators/run.sh` so quality evaluation is executable from the public toolchain"
                .to_string(),
        );
    }
    if evidence.fixtures_files == 0 {
        recommendations.push(
            "check in at least one fixture under `fixtures/` so evidence and regressions remain reviewable"
                .to_string(),
        );
    }
    if latest_diagnostic.is_none() {
        recommendations.push(
            "execute the block through the runtime at least once to populate latest diagnostics for `block doctor`"
                .to_string(),
        );
    }
    if warnings.iter().any(|warning| warning.contains("status")) {
        recommendations.push(
            "resolve active-status migration warnings before promoting the block".to_string(),
        );
    }
    recommendations
}

fn render_block_doctor_report(
    report: BlockDoctorReport,
    json_output: bool,
) -> Result<String, String> {
    if json_output {
        return serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render block doctor JSON: {error}"));
    }

    let mut lines = vec![
        format!("block doctor: {}", report.status),
        format!(
            "id: {}",
            report
                .block_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        ),
        format!("path: {}", report.path),
        format!("check_status: {}", report.check_status),
        format!(
            "evidence: tests={} examples={} evaluators={} fixtures={}",
            report.evidence.tests_files,
            report.evidence.examples_files,
            report.evidence.evaluators_files,
            report.evidence.fixtures_files
        ),
    ];
    if let Some(diagnostic) = &report.latest_diagnostic {
        lines.push(format!(
            "latest_diagnostic: {} ({})",
            diagnostic.execution_id, diagnostic.status
        ));
        if let Some(error_id) = &diagnostic.error_id {
            lines.push(format!("error_id: {error_id}"));
        }
    }
    for warning in &report.warnings {
        lines.push(format!("warning: {warning}"));
    }
    for error in &report.errors {
        lines.push(format!("error: {error}"));
    }
    for recommendation in &report.recommendations {
        lines.push(format!("next: {recommendation}"));
    }
    Ok(lines.join("\n"))
}

fn render_check_failure(
    json_output: bool,
    contract_path: &Path,
    block_id: Option<&str>,
    errors: &[String],
    warnings: &[String],
) -> Result<String, String> {
    if json_output {
        let payload = json!({
            "status": "error",
            "kind": "block",
            "path": contract_path.display().to_string(),
            "block_id": block_id,
            "warnings": warnings,
            "errors": errors,
        });
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("failed to render block check JSON: {error}"))?;
        Err(rendered)
    } else {
        let mut lines = vec![
            "block check: error".to_string(),
            format!("path: {}", contract_path.display()),
        ];
        if let Some(block_id) = block_id {
            lines.push(format!("id: {block_id}"));
        }
        for warning in warnings {
            lines.push(format!("warning: {warning}"));
        }
        for error in errors {
            lines.push(format!("error: {error}"));
        }
        Err(lines.join("\n"))
    }
}
