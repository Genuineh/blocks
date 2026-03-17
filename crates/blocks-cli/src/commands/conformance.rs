use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use blocks_runner_catalog::default_block_runner;
use blocks_runtime::{ExecutionContext, ExecutionEnvelope, generate_trace_id};
use serde::Serialize;
use serde_json::Value;

use crate::app::load_moc_manifest;
use crate::app::toolchain::resolve_descriptor_path;

use super::{bcl, block, moc, moc_bcl, pkg, runtime};

pub const BLOCKS_BCL_GATE_MODE_ENV: &str = "BLOCKS_BCL_GATE_MODE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BclGateMode {
    Off,
    Warn,
    Error,
}

#[derive(Default)]
struct ConformanceOptions {
    json: bool,
    check_against: Option<String>,
    gate_mode: Option<BclGateMode>,
    providers: Vec<String>,
    compat: bool,
    input_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ConformanceCase {
    name: String,
    status: String,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ConformanceFailure {
    case: String,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ConformanceReport {
    suite: String,
    status: String,
    target: String,
    cases_run: usize,
    failures: Vec<ConformanceFailure>,
    warnings: Vec<String>,
    artifacts: Vec<String>,
    cases: Vec<ConformanceCase>,
    gate_mode: Option<String>,
}

pub fn run_command(args: &[String]) -> Result<String, String> {
    match args {
        [suite, target] if suite == "block" => run_block_conformance(target, &[]),
        [suite, target, rest @ ..] if suite == "block" => run_block_conformance(target, rest),
        [suite, target] if suite == "package" => run_package_conformance(target, &[]),
        [suite, target, rest @ ..] if suite == "package" => run_package_conformance(target, rest),
        [suite, target] if suite == "runtime" => run_runtime_conformance(target, &[]),
        [suite, target, rest @ ..] if suite == "runtime" => run_runtime_conformance(target, rest),
        [suite, blocks_root, manifest_path] if suite == "moc" => {
            run_moc_conformance(blocks_root, manifest_path, &[])
        }
        [suite, blocks_root, manifest_path, rest @ ..] if suite == "moc" => {
            run_moc_conformance(blocks_root, manifest_path, rest)
        }
        [suite, target] if suite == "bcl" => run_bcl_conformance_target(target, &[]),
        [suite, target, rest @ ..]
            if suite == "bcl" && rest.first().is_none_or(|value| value.starts_with("--")) =>
        {
            run_bcl_conformance_target(target, rest)
        }
        [suite, blocks_root, source_path] if suite == "bcl" => {
            run_bcl_conformance_legacy(blocks_root, source_path, &[])
        }
        [suite, blocks_root, source_path, rest @ ..] if suite == "bcl" => {
            run_bcl_conformance_legacy(blocks_root, source_path, rest)
        }
        _ => Err(
            "usage: blocks conformance run block <block-root|block.yaml> [--json]\n       blocks conformance run package <package-root|package.yaml> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks conformance run runtime <block-root|block.yaml> [--host <sync-cli|tokio-service>]... [--input <json-file>] [--json]\n       blocks conformance run moc <blocks-root> <moc-root|moc.yaml> [--json]\n       blocks conformance run bcl <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--check-against <moc.yaml>] [--gate-mode <off|warn|error>] [--json]\n       blocks conformance run bcl <blocks-root> <moc-root|moc.bcl> [--check-against <moc.yaml>] [--gate-mode <off|warn|error>] [--json]".to_string(),
        ),
    }
}

fn run_block_conformance(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_conformance_options(args, false, false, false)?;
    let json_args = vec![String::from("--json")];
    let check = decode_json_result(block::check_command(target, &json_args), "block check")?;
    let test = decode_json_result(block::test_command(target, &json_args), "block test")?;
    let eval = decode_json_result(block::eval_command(target, &json_args), "block eval")?;

    let mut cases = vec![
        case_from_payload("block.check", &check),
        case_from_payload("block.test", &test),
        case_from_payload("block.eval", &eval),
    ];
    let mut failures = Vec::new();
    let mut warnings = collect_payload_warnings(&check);
    warnings.extend(collect_payload_warnings(&test));
    warnings.extend(collect_payload_warnings(&eval));

    extend_failures_from_payload(&mut failures, "block.check", &check);
    extend_failures_from_payload(&mut failures, "block.test", &test);
    extend_failures_from_payload(&mut failures, "block.eval", &eval);

    let evidence = test
        .get("evidence")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for field in [
        "tests_files",
        "examples_files",
        "evaluators_files",
        "fixtures_files",
    ] {
        if evidence.get(field).and_then(Value::as_u64).unwrap_or(0) == 0 {
            failures.push(ConformanceFailure {
                case: "block.evidence".to_string(),
                message: format!("expected non-empty block evidence directory for {field}"),
            });
        }
    }

    let status = if failures.is_empty() { "ok" } else { "error" };
    let report = ConformanceReport {
        suite: "block".to_string(),
        status: status.to_string(),
        target: target.to_string(),
        cases_run: cases.len(),
        failures,
        warnings,
        artifacts: vec![],
        cases: {
            cases.sort_by(|left, right| left.name.cmp(&right.name));
            cases
        },
        gate_mode: None,
    };

    render_conformance_report(report, options.json)
}

fn run_package_conformance(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_conformance_options(args, false, true, false)?;
    let package_root = resolve_package_root(target);

    let resolve = decode_json_result(
        pkg::run_command(&package_resolve_args(target, &options, false)),
        "pkg resolve",
    )?;
    let lock_first_output = pkg::run_command(&package_resolve_args(target, &options, true));
    let lock_first = decode_json_result(lock_first_output, "pkg resolve --lock")?;

    let mut cases = vec![
        case_from_payload("pkg.resolve", &resolve),
        case_from_payload("pkg.resolve.lock", &lock_first),
    ];
    let mut failures = Vec::new();
    let mut warnings = collect_payload_warnings(&resolve);
    warnings.extend(collect_payload_warnings(&lock_first));

    extend_failures_from_payload(&mut failures, "pkg.resolve", &resolve);
    extend_failures_from_payload(&mut failures, "pkg.resolve.lock", &lock_first);

    let lock_path = package_root.join("blocks.lock");
    let first_lock_contents =
        if lock_path.is_file() {
            Some(fs::read_to_string(&lock_path).map_err(|error| {
                format!("failed to read lockfile {}: {error}", lock_path.display())
            })?)
        } else {
            failures.push(ConformanceFailure {
                case: "pkg.resolve.lock".to_string(),
                message: format!("expected lockfile at {}", lock_path.display()),
            });
            None
        };

    let deterministic_case = if let Some(first_lock_contents) = first_lock_contents {
        let lock_second_output = pkg::run_command(&package_resolve_args(target, &options, true));
        let lock_second = decode_json_result(lock_second_output, "pkg resolve --lock (repeat)")?;
        extend_failures_from_payload(&mut failures, "pkg.resolve.repeat", &lock_second);
        warnings.extend(collect_payload_warnings(&lock_second));

        let second_lock_contents = fs::read_to_string(&lock_path).map_err(|error| {
            format!("failed to reread lockfile {}: {error}", lock_path.display())
        })?;
        if lock_first == lock_second && first_lock_contents == second_lock_contents {
            ConformanceCase {
                name: "pkg.resolve.repeat".to_string(),
                status: "ok".to_string(),
                message: "repeated package resolution produced identical JSON and lockfile output"
                    .to_string(),
            }
        } else {
            failures.push(ConformanceFailure {
                case: "pkg.resolve.repeat".to_string(),
                message: "repeated package resolution changed JSON output or lockfile bytes"
                    .to_string(),
            });
            ConformanceCase {
                name: "pkg.resolve.repeat".to_string(),
                status: "error".to_string(),
                message: "repeated package resolution changed JSON output or lockfile bytes"
                    .to_string(),
            }
        }
    } else {
        ConformanceCase {
            name: "pkg.resolve.repeat".to_string(),
            status: "error".to_string(),
            message: format!(
                "determinism check skipped because lockfile {} was not written",
                lock_path.display()
            ),
        }
    };
    cases.push(deterministic_case);

    let status = if failures.is_empty() { "ok" } else { "error" };
    let report = ConformanceReport {
        suite: "package".to_string(),
        status: status.to_string(),
        target: package_root.display().to_string(),
        cases_run: cases.len(),
        failures,
        warnings,
        artifacts: vec![lock_path.display().to_string()],
        cases,
        gate_mode: None,
    };

    render_conformance_report(report, options.json)
}

fn run_runtime_conformance(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_conformance_options(args, false, false, true)?;
    let (contract, _, block_root) = runtime::load_block_contract(target)?;
    let input = if let Some(path) = &options.input_path {
        crate::app::read_json_file(path)?
    } else {
        runtime::discover_runtime_input(&block_root)?.1
    };
    let diagnostics_root = runtime::default_runtime_diagnostics_root(&block_root);
    let hosts =
        runtime::instantiate_hosts(&selected_runtime_profiles(&options)?, &diagnostics_root);
    let runner = default_block_runner();
    let context = ExecutionContext {
        trace_id: Some(generate_trace_id()),
        moc_id: None,
    };

    let mut cases = Vec::new();
    let mut failures = Vec::new();
    let mut outputs = Vec::new();

    for host in hosts {
        let report = host.check_contract(&contract);
        let check_case_name = format!("runtime.check.{}", report.host_profile);
        let check_message = if report.status == "error" {
            report.errors.join("; ")
        } else if report.status == "warn" {
            report.warnings.join("; ")
        } else {
            "host is compatible with the block contract".to_string()
        };
        cases.push(ConformanceCase {
            name: check_case_name.clone(),
            status: report.status.clone(),
            message: check_message.clone(),
        });
        if report.status == "error" {
            failures.push(ConformanceFailure {
                case: check_case_name,
                message: check_message,
            });
            continue;
        }

        let profile = host.profile();
        let envelope = ExecutionEnvelope {
            contract: &contract,
            input: &input,
            context: &context,
        };
        match host.execute_envelope(&envelope, &runner) {
            Ok(result) => {
                cases.push(ConformanceCase {
                    name: format!("runtime.execute.{}", profile.as_str()),
                    status: "ok".to_string(),
                    message: format!(
                        "execution_id={} output validated by {}",
                        result.record.execution_id,
                        profile.as_str()
                    ),
                });
                outputs.push((profile.as_str().to_string(), result.output));
            }
            Err(error) => {
                failures.push(ConformanceFailure {
                    case: format!("runtime.execute.{}", profile.as_str()),
                    message: error.to_string(),
                });
                cases.push(ConformanceCase {
                    name: format!("runtime.execute.{}", profile.as_str()),
                    status: "error".to_string(),
                    message: error.to_string(),
                });
            }
        }
    }

    if outputs.len() > 1 {
        let first = &outputs[0].1;
        if outputs.iter().skip(1).all(|(_, output)| output == first) {
            cases.push(ConformanceCase {
                name: "runtime.output_parity".to_string(),
                status: "ok".to_string(),
                message: "all runtime host profiles produced identical output".to_string(),
            });
        } else {
            failures.push(ConformanceFailure {
                case: "runtime.output_parity".to_string(),
                message: "runtime host profiles produced different outputs".to_string(),
            });
            cases.push(ConformanceCase {
                name: "runtime.output_parity".to_string(),
                status: "error".to_string(),
                message: "runtime host profiles produced different outputs".to_string(),
            });
        }
    }

    let status = if failures.is_empty() { "ok" } else { "error" };
    let report = ConformanceReport {
        suite: "runtime".to_string(),
        status: status.to_string(),
        target: block_root.display().to_string(),
        cases_run: cases.len(),
        failures,
        warnings: Vec::new(),
        artifacts: vec![diagnostics_root.display().to_string()],
        cases,
        gate_mode: None,
    };

    render_conformance_report(report, options.json)
}

fn run_moc_conformance(
    blocks_root: &str,
    manifest_path: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_conformance_options(args, false, false, false)?;
    let manifest_path = resolve_descriptor_path(manifest_path, "moc.yaml");
    let manifest_path_string = manifest_path.display().to_string();
    let json_args = vec![String::from("--json")];
    let check = decode_json_result(
        moc::check_command(blocks_root, &manifest_path_string, &json_args),
        "moc check",
    )?;

    let mut cases = vec![case_from_payload("moc.check", &check)];
    let mut failures = Vec::new();
    let warnings = collect_payload_warnings(&check);
    extend_failures_from_payload(&mut failures, "moc.check", &check);

    let (manifest, _, _) = load_moc_manifest(&manifest_path_string)?;
    if manifest.has_validation_flow() {
        match moc::verify_command(blocks_root, &manifest_path_string, None) {
            Ok(message) => cases.push(ConformanceCase {
                name: "moc.verify".to_string(),
                status: "ok".to_string(),
                message,
            }),
            Err(message) => {
                failures.push(ConformanceFailure {
                    case: "moc.verify".to_string(),
                    message: message.clone(),
                });
                cases.push(ConformanceCase {
                    name: "moc.verify".to_string(),
                    status: "error".to_string(),
                    message,
                });
            }
        }
    }

    let status = if failures.is_empty() { "ok" } else { "error" };
    let report = ConformanceReport {
        suite: "moc".to_string(),
        status: status.to_string(),
        target: manifest_path_string,
        cases_run: cases.len(),
        failures,
        warnings,
        artifacts: vec![],
        cases,
        gate_mode: None,
    };

    render_conformance_report(report, options.json)
}

fn run_bcl_conformance_legacy(
    blocks_root: &str,
    source_path: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_conformance_options(args, true, false, false)?;
    let gate_mode = resolve_bcl_gate_mode(options.gate_mode)?;
    let source_path = resolve_descriptor_path(source_path, "moc.bcl");
    let source_path_string = source_path.display().to_string();
    let json_args = vec![String::from("--json")];
    let check = decode_json_result(
        moc_bcl::check_command(blocks_root, &source_path_string, &json_args),
        "moc bcl check",
    )?;

    let mut cases = vec![case_from_payload("moc.bcl.check", &check)];
    let mut failures = Vec::new();
    let mut warnings = collect_payload_warnings(&check);
    extend_failures_from_payload(&mut failures, "moc.bcl.check", &check);

    match moc_bcl::plan_command(blocks_root, &source_path_string, &json_args) {
        Ok(message) => cases.push(ConformanceCase {
            name: "moc.bcl.plan".to_string(),
            status: "ok".to_string(),
            message,
        }),
        Err(message) => {
            failures.push(ConformanceFailure {
                case: "moc.bcl.plan".to_string(),
                message: message.clone(),
            });
            cases.push(ConformanceCase {
                name: "moc.bcl.plan".to_string(),
                status: "error".to_string(),
                message,
            });
        }
    }

    let mut artifacts = vec![source_path_string.clone()];
    if let Some(check_against) = &options.check_against {
        artifacts.push(check_against.clone());
        if gate_mode == BclGateMode::Off {
            cases.push(ConformanceCase {
                name: "moc.bcl.parity".to_string(),
                status: "skipped".to_string(),
                message: format!(
                    "parity gate is disabled by {}=off; skipped parity against {}",
                    BLOCKS_BCL_GATE_MODE_ENV, check_against
                ),
            });
        } else {
            let emit_args = vec![String::from("--check-against"), check_against.clone()];
            match moc_bcl::emit_command(blocks_root, &source_path_string, &emit_args) {
                Ok(message) => cases.push(ConformanceCase {
                    name: "moc.bcl.parity".to_string(),
                    status: "ok".to_string(),
                    message,
                }),
                Err(message) => match gate_mode {
                    BclGateMode::Off => unreachable!(),
                    BclGateMode::Warn => {
                        warnings.push(message.clone());
                        cases.push(ConformanceCase {
                            name: "moc.bcl.parity".to_string(),
                            status: "warn".to_string(),
                            message,
                        });
                    }
                    BclGateMode::Error => {
                        failures.push(ConformanceFailure {
                            case: "moc.bcl.parity".to_string(),
                            message: message.clone(),
                        });
                        cases.push(ConformanceCase {
                            name: "moc.bcl.parity".to_string(),
                            status: "error".to_string(),
                            message,
                        });
                    }
                },
            }
        }
    } else if gate_mode == BclGateMode::Off {
        cases.push(ConformanceCase {
            name: "moc.bcl.parity".to_string(),
            status: "skipped".to_string(),
            message: format!(
                "parity gate is disabled by {}=off and no --check-against manifest was provided",
                BLOCKS_BCL_GATE_MODE_ENV
            ),
        });
    } else {
        let message = String::from(
            "BCL conformance requires --check-against <moc.yaml> when gate mode is warn or error",
        );
        failures.push(ConformanceFailure {
            case: "moc.bcl.parity".to_string(),
            message: message.clone(),
        });
        cases.push(ConformanceCase {
            name: "moc.bcl.parity".to_string(),
            status: "error".to_string(),
            message,
        });
    }

    let status = if failures.is_empty() { "ok" } else { "error" };
    let report = ConformanceReport {
        suite: "bcl".to_string(),
        status: status.to_string(),
        target: source_path_string,
        cases_run: cases.len(),
        failures,
        warnings,
        artifacts,
        cases,
        gate_mode: Some(gate_mode.as_str().to_string()),
    };

    render_conformance_report(report, options.json)
}

fn run_bcl_conformance_target(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_conformance_options(args, true, true, false)?;
    let gate_mode = resolve_bcl_gate_mode(options.gate_mode)?;
    let mut shared_args = Vec::new();
    for provider in &options.providers {
        shared_args.push("--provider".to_string());
        shared_args.push(provider.clone());
    }
    if options.compat {
        shared_args.push("--compat".to_string());
    }
    let mut json_args = shared_args.clone();
    json_args.push("--json".to_string());

    let check = decode_json_result(bcl::check_command(target, &json_args), "bcl check")?;
    let graph = decode_json_result(bcl::graph_command(target, &json_args), "bcl graph")?;
    let build = decode_json_result(bcl::build_command(target, &json_args), "bcl build")?;

    let mut cases = vec![
        case_from_payload("bcl.check", &check),
        case_from_payload("bcl.graph", &graph),
        case_from_payload("bcl.build", &build),
    ];
    let mut failures = Vec::new();
    let mut warnings = collect_payload_warnings(&check);
    warnings.extend(collect_payload_warnings(&graph));
    warnings.extend(collect_payload_warnings(&build));

    extend_failures_from_payload(&mut failures, "bcl.check", &check);
    extend_failures_from_payload(&mut failures, "bcl.graph", &graph);
    extend_failures_from_payload(&mut failures, "bcl.build", &build);

    let mut artifacts = build
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|artifact| artifact.get("path").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if let Some(check_against) = &options.check_against {
        artifacts.push(check_against.clone());
        let build_artifact = artifacts
            .first()
            .cloned()
            .ok_or_else(|| "bcl build did not report an artifact path".to_string())?;
        if gate_mode == BclGateMode::Off {
            cases.push(ConformanceCase {
                name: "bcl.parity".to_string(),
                status: "skipped".to_string(),
                message: format!(
                    "parity gate is disabled by {}=off; skipped parity against {}",
                    BLOCKS_BCL_GATE_MODE_ENV, check_against
                ),
            });
        } else {
            let emitted = fs::read_to_string(&build_artifact).map_err(|error| {
                format!(
                    "failed to read bcl build artifact {}: {error}",
                    build_artifact
                )
            })?;
            match blocks_bcl::check_against_file(&emitted, check_against) {
                Ok(()) => cases.push(ConformanceCase {
                    name: "bcl.parity".to_string(),
                    status: "ok".to_string(),
                    message: format!("build artifact matched {}", check_against),
                }),
                Err(message) => match gate_mode {
                    BclGateMode::Off => unreachable!(),
                    BclGateMode::Warn => {
                        warnings.push(message.clone());
                        cases.push(ConformanceCase {
                            name: "bcl.parity".to_string(),
                            status: "warn".to_string(),
                            message,
                        });
                    }
                    BclGateMode::Error => {
                        failures.push(ConformanceFailure {
                            case: "bcl.parity".to_string(),
                            message: message.clone(),
                        });
                        cases.push(ConformanceCase {
                            name: "bcl.parity".to_string(),
                            status: "error".to_string(),
                            message,
                        });
                    }
                },
            }
        }
    } else if gate_mode == BclGateMode::Off {
        cases.push(ConformanceCase {
            name: "bcl.parity".to_string(),
            status: "skipped".to_string(),
            message: format!(
                "parity gate is disabled by {}=off and no --check-against manifest was provided",
                BLOCKS_BCL_GATE_MODE_ENV
            ),
        });
    } else {
        let message = String::from(
            "BCL conformance requires --check-against <moc.yaml> when gate mode is warn or error",
        );
        failures.push(ConformanceFailure {
            case: "bcl.parity".to_string(),
            message: message.clone(),
        });
        cases.push(ConformanceCase {
            name: "bcl.parity".to_string(),
            status: "error".to_string(),
            message,
        });
    }

    let status = if failures.is_empty() { "ok" } else { "error" };
    let report = ConformanceReport {
        suite: "bcl".to_string(),
        status: status.to_string(),
        target: target.to_string(),
        cases_run: cases.len(),
        failures,
        warnings,
        artifacts,
        cases,
        gate_mode: Some(gate_mode.as_str().to_string()),
    };

    render_conformance_report(report, options.json)
}

fn parse_conformance_options(
    args: &[String],
    allow_bcl_options: bool,
    allow_package_options: bool,
    allow_runtime_options: bool,
) -> Result<ConformanceOptions, String> {
    let mut options = ConformanceOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--check-against" if allow_bcl_options => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--check-against requires a value".to_string())?;
                options.check_against = Some(value.clone());
                index += 2;
            }
            "--gate-mode" if allow_bcl_options => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--gate-mode requires a value".to_string())?;
                options.gate_mode = Some(parse_bcl_gate_mode(value)?);
                index += 2;
            }
            "--provider" if allow_package_options => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--provider requires a value".to_string())?;
                options.providers.push(value.clone());
                index += 2;
            }
            "--compat" if allow_package_options => {
                options.compat = true;
                index += 1;
            }
            "--host" if allow_runtime_options => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--host requires a value".to_string())?;
                options.providers.push(value.clone());
                index += 2;
            }
            "--input" if allow_runtime_options => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--input requires a value".to_string())?;
                options.input_path = Some(value.clone());
                index += 2;
            }
            other => return Err(format!("unknown option for conformance run: {other}")),
        }
    }
    Ok(options)
}

fn package_resolve_args(target: &str, options: &ConformanceOptions, lock: bool) -> Vec<String> {
    let mut args = vec!["resolve".to_string(), target.to_string()];
    if options.compat {
        args.push("--compat".to_string());
    }
    for provider in &options.providers {
        args.push("--provider".to_string());
        args.push(provider.clone());
    }
    if lock {
        args.push("--lock".to_string());
    }
    args.push("--json".to_string());
    args
}

fn selected_runtime_profiles(
    options: &ConformanceOptions,
) -> Result<Vec<blocks_runtime::HostProfile>, String> {
    if options.providers.is_empty() {
        return Ok(vec![
            blocks_runtime::HostProfile::SyncCli,
            blocks_runtime::HostProfile::TokioService,
        ]);
    }
    options
        .providers
        .iter()
        .map(|raw| blocks_runtime::HostProfile::parse(raw))
        .collect()
}

fn resolve_package_root(target: &str) -> PathBuf {
    let path = PathBuf::from(target);
    if path
        .file_name()
        .and_then(|item| item.to_str())
        .is_some_and(|name| name == "package.yaml")
    {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        path
    }
}

fn parse_bcl_gate_mode(raw: &str) -> Result<BclGateMode, String> {
    match raw {
        "off" => Ok(BclGateMode::Off),
        "warn" => Ok(BclGateMode::Warn),
        "error" => Ok(BclGateMode::Error),
        other => Err(format!("unsupported BCL gate mode: {other}")),
    }
}

fn resolve_bcl_gate_mode(explicit: Option<BclGateMode>) -> Result<BclGateMode, String> {
    if let Some(mode) = explicit {
        return Ok(mode);
    }
    match env::var(BLOCKS_BCL_GATE_MODE_ENV) {
        Ok(value) => parse_bcl_gate_mode(&value),
        Err(_) => Ok(BclGateMode::Warn),
    }
}

fn decode_json_result(result: Result<String, String>, label: &str) -> Result<Value, String> {
    let rendered = match result {
        Ok(output) => output,
        Err(output) => output,
    };
    serde_json::from_str(&rendered)
        .map_err(|error| format!("failed to parse {label} JSON output: {error}\n{rendered}"))
}

fn case_from_payload(name: &str, payload: &Value) -> ConformanceCase {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("error")
        .to_string();
    let message = first_payload_message(payload);
    ConformanceCase {
        name: name.to_string(),
        status,
        message,
    }
}

fn first_payload_message(payload: &Value) -> String {
    if let Some(message) = payload
        .get("failures")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
    {
        return message.to_string();
    }
    if let Some(message) = payload
        .get("errors")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(Value::as_str)
    {
        return message.to_string();
    }
    if let Some(message) = payload
        .get("warnings")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(Value::as_str)
    {
        return message.to_string();
    }

    payload
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("ok")
        .to_string()
}

fn collect_payload_warnings(payload: &Value) -> Vec<String> {
    payload
        .get("warnings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect()
}

fn extend_failures_from_payload(
    failures: &mut Vec<ConformanceFailure>,
    case: &str,
    payload: &Value,
) {
    let mut recorded_structured_failure = false;
    if let Some(values) = payload.get("errors").and_then(Value::as_array) {
        for value in values {
            if let Some(message) = value.as_str() {
                recorded_structured_failure = true;
                failures.push(ConformanceFailure {
                    case: case.to_string(),
                    message: message.to_string(),
                });
            }
        }
    }
    if let Some(values) = payload.get("failures").and_then(Value::as_array) {
        for value in values {
            if let Some(message) = value.get("message").and_then(Value::as_str) {
                recorded_structured_failure = true;
                failures.push(ConformanceFailure {
                    case: case.to_string(),
                    message: message.to_string(),
                });
            }
        }
    }
    if !recorded_structured_failure
        && payload
            .get("status")
            .and_then(Value::as_str)
            .is_some_and(|status| status == "error")
    {
        if let Some(message) = payload.get("message").and_then(Value::as_str) {
            failures.push(ConformanceFailure {
                case: case.to_string(),
                message: message.to_string(),
            });
        }
    }
}

fn render_conformance_report(
    report: ConformanceReport,
    json_output: bool,
) -> Result<String, String> {
    if json_output {
        let rendered = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render conformance JSON: {error}"))?;
        return if report.status == "error" {
            Err(rendered)
        } else {
            Ok(rendered)
        };
    }

    let mut lines = vec![
        format!("conformance {}: {}", report.suite, report.status),
        format!("target: {}", report.target),
        format!("cases_run: {}", report.cases_run),
    ];
    if let Some(gate_mode) = &report.gate_mode {
        lines.push(format!("gate_mode: {gate_mode}"));
    }
    for case in &report.cases {
        lines.push(format!(
            "case {}: {} ({})",
            case.name, case.status, case.message
        ));
    }
    for warning in &report.warnings {
        lines.push(format!("warning: {warning}"));
    }
    for failure in &report.failures {
        lines.push(format!("error: {}: {}", failure.case, failure.message));
    }

    let rendered = lines.join("\n");
    if report.status == "error" {
        Err(rendered)
    } else {
        Ok(rendered)
    }
}

impl BclGateMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
