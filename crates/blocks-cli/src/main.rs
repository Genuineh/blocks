use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitCode;

use blocks_moc::{ExecutionPlan, MocComposer, MocError, MocManifest, MocType};
use blocks_registry::Registry;
use blocks_runner_catalog::default_block_runner;
use blocks_runtime::{
    BlockRunner, DiagnosticArtifact, DiagnosticEvent, ExecutionContext, Runtime, generate_trace_id,
    read_diagnostic_artifact, read_diagnostic_events,
};
use serde_json::{Value, json};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    match args.as_slice() {
        [command, root] if command == "list" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            Ok(registry
                .list()
                .iter()
                .map(|block| block.contract.id.as_str())
                .collect::<Vec<_>>()
                .join("\n"))
        }
        [command, root, block_id] if command == "show" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let Some(block) = registry.get(block_id) else {
                return Err(format!("block not found: {block_id}"));
            };

            let mut lines = vec![format!("id: {}", block.contract.id)];
            if let Some(name) = &block.contract.name {
                lines.push(format!("name: {name}"));
            }
            lines.push(format!("contract: {}", block.contract_path.display()));
            lines.push(format!("implementation: {}", block.implementation_path.display()));
            if let Some(implementation) = &block.contract.implementation {
                lines.push(format!("implementation_kind: {:?}", implementation.kind));
                lines.push(format!("implementation_target: {:?}", implementation.target));
            }
            Ok(lines.join("\n"))
        }
        [command, root, query] if command == "search" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            Ok(registry
                .search(query)
                .iter()
                .map(|block| block.contract.id.as_str())
                .collect::<Vec<_>>()
                .join("\n"))
        }
        [command, root, block_id, input_path] if command == "run" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let Some(block) = registry.get(block_id) else {
                return Err(format!("block not found: {block_id}"));
            };
            let input = read_json_file(input_path)?;
            let runner = default_block_runner();
            let runtime = Runtime::with_diagnostics_root(resolve_diagnostics_root(root));
            let result = runtime
                .execute(&block.contract, &input, &runner)
                .map_err(|error| error.to_string())?;

            serde_json::to_string_pretty(&result.output)
                .map_err(|error| format!("failed to render output JSON: {error}"))
        }
        [command, subcommand, root, block_id] if command == "block" && subcommand == "diagnose" => {
            block_diagnose_command(root, block_id, &[])
        }
        [command, subcommand, root, block_id, rest @ ..]
            if command == "block" && subcommand == "diagnose" =>
        {
            block_diagnose_command(root, block_id, rest)
        }
        [command, subcommand, root, manifest_path] if command == "moc" && subcommand == "validate" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
            validate_moc_manifest(&manifest, moc_root, mocs_root)?;
            let mut details = vec![format!("type={}", manifest.moc_type)];
            if let Some(mode) = manifest.backend_mode {
                details.push(format!("backend_mode={mode}"));
            }

            if manifest.has_validation_flow() {
                let plan = MocComposer::new()
                    .plan(&manifest, &registry)
                    .map_err(|error| error.to_string())?;
                details.push(format!("flow={}", plan.flow_id));
                details.push(format!("steps={}", plan.steps.len()));
            } else {
                details.push("descriptor_only=true".to_string());
            }

            Ok(format!("valid: {} ({}) {}", manifest.id, manifest_path, details.join(" ")))
        }
        [command, subcommand, root, manifest_path] if command == "moc" && subcommand == "run" => {
            run_moc_command(root, manifest_path, None)
        }
        [command, subcommand, root, manifest_path, input_path] if command == "moc" && subcommand == "run" => {
            run_moc_command(root, manifest_path, Some(input_path))
        }
        [command, subcommand, root, manifest_path] if command == "moc" && subcommand == "verify" => {
            verify_moc_command(root, manifest_path, None)
        }
        [command, subcommand, root, manifest_path, input_path]
            if command == "moc" && subcommand == "verify" =>
        {
            verify_moc_command(root, manifest_path, Some(input_path))
        }
        [command, subcommand, root, manifest_path] if command == "moc" && subcommand == "dev" => {
            dev_moc_command(root, manifest_path)
        }
        [command, subcommand, root, manifest_path] if command == "moc" && subcommand == "diagnose" => {
            moc_diagnose_command(root, manifest_path, &[])
        }
        [command, subcommand, root, manifest_path, rest @ ..]
            if command == "moc" && subcommand == "diagnose" =>
        {
            moc_diagnose_command(root, manifest_path, rest)
        }
        _ => Err(
            "usage: blocks <list|show|search> <blocks-root> [query|block-id]\n       blocks run <blocks-root> <block-id> <input-json-file>\n       blocks block diagnose <blocks-root> <block-id> [--latest|--execution-id <id>] [--json]\n       blocks moc validate <blocks-root> <moc-yaml>\n       blocks moc run <blocks-root> <moc-yaml> [input-json-file]\n       blocks moc verify <blocks-root> <moc-yaml> [input-json-file]\n       blocks moc dev <blocks-root> <moc-yaml>\n       blocks moc diagnose <blocks-root> <moc-yaml> [--trace-id <id>] [--json]"
                .to_string(),
        ),
    }
}

fn load_moc_manifest(manifest_path: &str) -> Result<(MocManifest, &Path, &Path), String> {
    let manifest_source = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read moc manifest {manifest_path}: {error}"))?;
    let manifest = MocManifest::from_yaml_str(&manifest_source)
        .map_err(|error| format!("failed to load moc manifest {manifest_path}: {error}"))?;
    let moc_root = Path::new(manifest_path)
        .parent()
        .ok_or_else(|| format!("invalid moc manifest path: {manifest_path}"))?;
    let mocs_root = moc_root.parent().unwrap_or(moc_root);

    Ok((manifest, moc_root, mocs_root))
}

fn validate_moc_manifest(
    manifest: &MocManifest,
    moc_root: &Path,
    mocs_root: &Path,
) -> Result<(), String> {
    manifest
        .validate_layout(moc_root)
        .map_err(|error| error.to_string())?;
    manifest
        .validate_dependencies(mocs_root)
        .map_err(|error| error.to_string())
}

fn block_diagnose_command(
    blocks_root: &str,
    block_id: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_block_diagnose_options(args)?;
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

fn moc_diagnose_command(
    blocks_root: &str,
    manifest_path: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_moc_diagnose_options(args)?;
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    validate_moc_manifest(&manifest, moc_root, mocs_root)?;

    let diagnostics_root = resolve_diagnostics_root(blocks_root);
    let events = read_diagnostic_events(&diagnostics_root)?;
    let trace_id = match options.trace_id {
        Some(trace_id) => trace_id,
        None => select_latest_trace_id_for_moc(&events, &manifest).ok_or_else(|| {
            format!(
                "no diagnostics found for moc {}. run `blocks moc verify` first",
                manifest.id
            )
        })?,
    };

    let trace_events: Vec<DiagnosticEvent> = events
        .into_iter()
        .filter(|event| event.trace_id.as_deref() == Some(trace_id.as_str()))
        .collect();
    if trace_events.is_empty() {
        return Err(format!("no diagnostics found for trace_id {trace_id}"));
    }

    let mut artifacts = Vec::new();
    for event in &trace_events {
        if event.event == "block.execution.failure" {
            if let Some(artifact) =
                read_diagnostic_artifact(&diagnostics_root, &event.execution_id)?
            {
                artifacts.push(artifact);
            }
        }
    }

    if options.json {
        return serde_json::to_string_pretty(&json!({
            "moc_id": manifest.id,
            "manifest_path": manifest_path,
            "trace_id": trace_id,
            "diagnostics_root": diagnostics_root,
            "events": trace_events,
            "artifacts": artifacts
        }))
        .map_err(|error| format!("failed to render diagnostic JSON: {error}"));
    }

    render_moc_diagnose_human(
        &manifest.id,
        &trace_id,
        diagnostics_root.as_path(),
        &trace_events,
        &artifacts,
    )
}

#[derive(Default)]
struct BlockDiagnoseOptions {
    execution_id: Option<String>,
    json: bool,
}

fn parse_block_diagnose_options(args: &[String]) -> Result<BlockDiagnoseOptions, String> {
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

#[derive(Default)]
struct MocDiagnoseOptions {
    trace_id: Option<String>,
    json: bool,
}

fn parse_moc_diagnose_options(args: &[String]) -> Result<MocDiagnoseOptions, String> {
    let mut options = MocDiagnoseOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--trace-id" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--trace-id requires a value".to_string())?;
                options.trace_id = Some(value.clone());
                index += 2;
            }
            other => {
                return Err(format!("unknown option for moc diagnose: {other}"));
            }
        }
    }
    Ok(options)
}

fn select_latest_execution_id(events: &[DiagnosticEvent]) -> Option<String> {
    events
        .iter()
        .max_by_key(|event| event.timestamp_ms)
        .map(|event| event.execution_id.clone())
}

fn select_latest_trace_id_for_moc(
    events: &[DiagnosticEvent],
    manifest: &MocManifest,
) -> Option<String> {
    let direct_moc_match = events
        .iter()
        .filter(|event| event.moc_id.as_deref() == Some(manifest.id.as_str()))
        .filter_map(|event| {
            event
                .trace_id
                .clone()
                .map(|trace_id| (event.timestamp_ms, trace_id))
        })
        .max_by_key(|(timestamp, _)| *timestamp)
        .map(|(_, trace_id)| trace_id);
    if direct_moc_match.is_some() {
        return direct_moc_match;
    }

    events
        .iter()
        .filter(|event| {
            event.moc_id.is_none() || event.moc_id.as_deref() == Some(manifest.id.as_str())
        })
        .filter(|event| {
            manifest
                .uses
                .blocks
                .iter()
                .any(|block| block == &event.block_id)
        })
        .filter_map(|event| {
            event
                .trace_id
                .clone()
                .map(|trace_id| (event.timestamp_ms, trace_id))
        })
        .max_by_key(|(timestamp, _)| *timestamp)
        .map(|(_, trace_id)| trace_id)
}

fn render_block_diagnose_human(
    block_id: &str,
    execution_id: &str,
    diagnostics_root: &Path,
    events: &[DiagnosticEvent],
    artifact: Option<&DiagnosticArtifact>,
) -> Result<String, String> {
    let last_event = events
        .last()
        .ok_or_else(|| format!("no diagnostics found for block {block_id}"))?;
    let status = if last_event.event == "block.execution.failure" {
        "failure"
    } else {
        "success"
    };
    let mut lines = vec![
        format!("block: {block_id}"),
        format!("execution_id: {execution_id}"),
        format!("status: {status}"),
        format!("events: {}", events.len()),
        format!("diagnostics_root: {}", diagnostics_root.display()),
    ];
    if let Some(trace_id) = &last_event.trace_id {
        lines.push(format!("trace_id: {trace_id}"));
    }
    if let Some(duration_ms) = last_event.duration_ms {
        lines.push(format!("duration_ms: {duration_ms}"));
    }
    if let Some(error_id) = &last_event.error_id {
        lines.push(format!("error_id: {error_id}"));
    }
    if artifact.is_some() {
        lines.push(format!(
            "artifact: {}/artifacts/{}.json",
            diagnostics_root.display(),
            execution_id
        ));
    }
    Ok(lines.join("\n"))
}

fn render_moc_diagnose_human(
    moc_id: &str,
    trace_id: &str,
    diagnostics_root: &Path,
    events: &[DiagnosticEvent],
    artifacts: &[DiagnosticArtifact],
) -> Result<String, String> {
    let first_event = events
        .first()
        .ok_or_else(|| format!("no diagnostics found for trace_id {trace_id}"))?;
    let last_event = events
        .last()
        .ok_or_else(|| format!("no diagnostics found for trace_id {trace_id}"))?;
    let total_duration = last_event
        .timestamp_ms
        .saturating_sub(first_event.timestamp_ms);
    let failure_count = events
        .iter()
        .filter(|event| event.event == "block.execution.failure")
        .count();

    let mut lines = vec![
        format!("moc: {moc_id}"),
        format!("trace_id: {trace_id}"),
        format!("events: {}", events.len()),
        format!("failures: {failure_count}"),
        format!("duration_ms: {total_duration}"),
        format!("diagnostics_root: {}", diagnostics_root.display()),
    ];
    if !artifacts.is_empty() {
        lines.push(format!("artifacts: {}", artifacts.len()));
    }
    Ok(lines.join("\n"))
}

fn run_moc_command(
    blocks_root: &str,
    manifest_path: &str,
    input_path: Option<&str>,
) -> Result<String, String> {
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    validate_moc_manifest(&manifest, moc_root, mocs_root)?;

    if let Some(cargo_manifest) = resolve_rust_backend_launcher(&manifest, moc_root) {
        return run_real_rust_backend(&cargo_manifest, input_path);
    }

    if let Some(cargo_manifest) = resolve_frontend_host_launcher(&manifest, moc_root) {
        return run_real_frontend_host(&cargo_manifest);
    }

    if let Some(preview_path) = resolve_frontend_preview(&manifest, moc_root) {
        return Ok(format!("frontend preview: {}", preview_path.display()));
    }

    if manifest.has_validation_flow() {
        let (output, trace_id) =
            execute_moc_flow_runtime_wrapper(blocks_root, &manifest, moc_root, input_path)
                .map_err(|error| render_moc_verify_error(manifest_path, &error))?;
        let rendered = serde_json::to_string_pretty(&output)
            .map_err(|error| format!("failed to render output JSON: {error}"))?;
        return Ok(format!("{rendered}\ntrace_id: {trace_id}"));
    }

    Err(format!(
        "moc run requires a real launcher or preview; use `moc verify` for verification flows ({})",
        manifest.id
    ))
}

fn verify_moc_command(
    blocks_root: &str,
    manifest_path: &str,
    input_path: Option<&str>,
) -> Result<String, String> {
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    validate_moc_manifest(&manifest, moc_root, mocs_root)?;
    let (output, trace_id) =
        execute_moc_flow_runtime_wrapper(blocks_root, &manifest, moc_root, input_path)
            .map_err(|error| render_moc_verify_error(manifest_path, &error))?;
    let rendered = serde_json::to_string_pretty(&output)
        .map_err(|error| format!("failed to render output JSON: {error}"))?;
    Ok(format!("{rendered}\ntrace_id: {trace_id}"))
}

fn execute_moc_flow_runtime_wrapper(
    blocks_root: &str,
    manifest: &MocManifest,
    moc_root: &Path,
    input_path: Option<&str>,
) -> Result<(Value, String), MocError> {
    if !manifest.has_validation_flow() {
        return Err(MocError::ValidationFlowNotConfigured);
    }

    let registry = Registry::load_from_root(blocks_root)
        .map_err(|error| MocError::InvalidDescriptor(error.to_string()))?;
    let plan = MocComposer::new().plan(manifest, &registry)?;
    let input = match input_path {
        Some(path) => read_json_file(path).map_err(MocError::InvalidDescriptor)?,
        None => read_default_moc_input(moc_root).map_err(MocError::InvalidDescriptor)?,
    };
    let runner = default_block_runner();
    let trace_id = generate_trace_id();
    let runtime = Runtime::with_diagnostics_root(resolve_diagnostics_root(blocks_root));
    let output = execute_validation_plan(
        &plan,
        manifest.id.as_str(),
        &registry,
        &input,
        &runner,
        &runtime,
        &trace_id,
    )?;
    Ok((output, trace_id))
}

fn dev_moc_command(blocks_root: &str, manifest_path: &str) -> Result<String, String> {
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    let workspace_root = resolve_browser_preview_root(blocks_root, mocs_root);
    validate_moc_manifest(&manifest, moc_root, mocs_root)?;

    match manifest.moc_type {
        MocType::RustLib => {
            let cargo_manifest = resolve_rust_lib_manifest(moc_root).ok_or_else(|| {
                format!("rust_lib dev requires Cargo.toml in {}", moc_root.display())
            })?;
            run_rust_lib_dev(&cargo_manifest)
        }
        MocType::FrontendLib => {
            if let Some(preview_path) = resolve_frontend_lib_preview(moc_root) {
                let mut lines = vec![format!("frontend lib preview: {}", preview_path.display())];
                lines.extend(render_browser_preview_lines(&preview_path, &workspace_root));
                return Ok(lines.join("\n"));
            }

            let entry_path = moc_root.join(&manifest.entry);
            if entry_path.is_file() {
                Ok(format!("frontend lib source: {}", entry_path.display()))
            } else {
                Err(format!(
                    "frontend_lib dev requires preview/index.html or a valid entry file; missing {}",
                    entry_path.display()
                ))
            }
        }
        MocType::FrontendApp => {
            let mut lines = vec![format!("frontend app dev: {}", manifest.id)];

            if let Some(preview_path) = resolve_frontend_preview(&manifest, moc_root) {
                lines.push(format!("web preview: {}", preview_path.display()));
                lines.extend(render_browser_preview_lines(&preview_path, &workspace_root));
            }

            if let Some(cargo_manifest) = resolve_frontend_host_launcher(&manifest, moc_root) {
                lines.push(format!(
                    "linux app: cargo run --manifest-path {}",
                    cargo_manifest.display()
                ));
                lines.push(format!(
                    "linux app (headless probe): cargo --offline run --manifest-path {} -- --headless-probe",
                    cargo_manifest.display()
                ));
            }

            if lines.len() == 1 {
                Err(format!(
                    "frontend_app dev requires preview/index.html or src-tauri/Cargo.toml; {} has neither",
                    manifest.id
                ))
            } else {
                Ok(lines.join("\n"))
            }
        }
        MocType::BackendApp => Err(format!(
            "moc dev is for library and frontend preview workflows; use moc run for {}",
            manifest.id
        )),
    }
}

fn execute_validation_plan(
    plan: &ExecutionPlan,
    moc_id: &str,
    registry: &Registry,
    input: &Value,
    runner: &impl BlockRunner,
    runtime: &Runtime,
    trace_id: &str,
) -> Result<Value, MocError> {
    let mut step_outputs = BTreeMap::new();
    let context = ExecutionContext {
        trace_id: Some(trace_id.to_string()),
        moc_id: Some(moc_id.to_string()),
    };

    for step in &plan.steps {
        let step_input = step.build_input(input, &step_outputs)?;
        let block = registry
            .get(&step.block)
            .ok_or_else(|| MocError::UnknownBlock(step.block.clone()))?;
        let result = runtime
            .execute_with_context(
                &block.contract,
                &Value::Object(step_input),
                runner,
                &context,
            )
            .map_err(|error| {
                MocError::InvalidDescriptor(format!(
                    "runtime execution failed for step {} (block {}): {}",
                    step.id, step.block, error
                ))
            })?;

        step_outputs.insert(step.id.clone(), result.output);
    }

    step_outputs.remove(&plan.last_step_id).ok_or_else(|| {
        MocError::InvalidDescriptor(format!(
            "missing output for final step: {}",
            plan.last_step_id
        ))
    })
}

fn render_moc_verify_error(manifest_path: &str, error: &MocError) -> String {
    let detail = match error {
        MocError::MissingBind {
            flow_id,
            step_id,
            field,
        } => format!(
            "{error}. verification.flows[{flow_id}] step `{step_id}` is missing a bind for required field `{step_id}.{field}`. Add a bind entry under that flow before running verify."
        ),
        MocError::TypeMismatch {
            flow_id,
            step_id,
            bind_index,
            from,
            to,
            expected,
            actual,
        } => format!(
            "{error}. verification.flows[{flow_id}] step `{step_id}` bind #{bind_index} (`{from}` -> `{to}`) uses the wrong source type. Fix that bind so the source matches the target field (expected {expected}, got {actual})."
        ),
        MocError::InvalidReference {
            flow_id,
            step_id,
            bind_index,
            from,
            to,
            reference,
        } => format!(
            "{error}. verification.flows[{flow_id}] step `{step_id}` bind #{bind_index} (`{from}` -> `{to}`) uses invalid reference `{reference}`. References must use `input.<field>` or `<step-id>.<field>`, the source step must run earlier, and the referenced field must exist in the verify input or previous step output."
        ),
        MocError::UnknownBlock(block_id) => format!(
            "{error}. Ensure `{block_id}` exists under blocks/ and is declared in uses.blocks."
        ),
        MocError::ValidationFlowNotConfigured => format!(
            "{error}. Add verification.entry_flow and verification.flows, or use moc run with a real launcher instead."
        ),
        MocError::EntryFlowNotFound(flow_id) => {
            format!("{error}. Define a flow with id `{flow_id}` under verification.flows.")
        }
        MocError::EmptyFlow(flow_id) => {
            format!("{error}. Add at least one step to verification.flows[{flow_id}].")
        }
        MocError::InvalidDescriptor(message) => format!("{error}. Descriptor detail: {message}"),
        MocError::ManifestParse(_) => error.to_string(),
    };

    format!("moc verify failed for {manifest_path}: {detail}")
}

fn resolve_rust_backend_launcher(manifest: &MocManifest, moc_root: &Path) -> Option<PathBuf> {
    if manifest.moc_type != MocType::BackendApp
        || manifest.language != "rust"
        || manifest.entry.trim().is_empty()
    {
        return None;
    }

    let entry_path = moc_root.join(&manifest.entry);
    let backend_root = entry_path.parent()?.parent()?;
    let cargo_manifest = backend_root.join("Cargo.toml");

    cargo_manifest.is_file().then_some(cargo_manifest)
}

fn resolve_rust_lib_manifest(moc_root: &Path) -> Option<PathBuf> {
    let cargo_manifest = moc_root.join("Cargo.toml");
    cargo_manifest.is_file().then_some(cargo_manifest)
}

fn run_real_rust_backend(
    cargo_manifest: &Path,
    input_path: Option<&str>,
) -> Result<String, String> {
    let mut command = Command::new("cargo");
    command
        .arg("run")
        .arg("--manifest-path")
        .arg(cargo_manifest);
    if let Some(input_path) = input_path {
        command.arg("--").arg(input_path);
    }

    let output = command.output().map_err(|error| {
        format!(
            "failed to launch moc backend {}: {error}",
            cargo_manifest.display()
        )
    })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!(
                "moc backend failed: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ))
        } else {
            Err(stderr)
        }
    }
}

fn run_rust_lib_dev(cargo_manifest: &Path) -> Result<String, String> {
    let output = Command::new("cargo")
        .arg("test")
        .arg("--manifest-path")
        .arg(cargo_manifest)
        .output()
        .map_err(|error| {
            format!(
                "failed to run rust_lib dev workflow {}: {error}",
                cargo_manifest.display()
            )
        })?;

    if output.status.success() {
        Ok(format!("rust lib dev ok: {}", cargo_manifest.display()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!(
                "rust lib dev failed: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ))
        } else {
            Err(stderr)
        }
    }
}

fn resolve_frontend_preview(manifest: &MocManifest, moc_root: &Path) -> Option<PathBuf> {
    if manifest.moc_type != MocType::FrontendApp || manifest.language != "tauri_ts" {
        return None;
    }

    let preview_index = moc_root.join("preview").join("index.html");
    if preview_index.is_file() {
        return Some(preview_index);
    }

    let preview_html = moc_root.join("preview.html");
    preview_html.is_file().then_some(preview_html)
}

fn resolve_frontend_lib_preview(moc_root: &Path) -> Option<PathBuf> {
    let preview_index = moc_root.join("preview").join("index.html");
    if preview_index.is_file() {
        return Some(preview_index);
    }

    let preview_html = moc_root.join("preview.html");
    preview_html.is_file().then_some(preview_html)
}

fn resolve_frontend_host_launcher(manifest: &MocManifest, moc_root: &Path) -> Option<PathBuf> {
    if manifest.moc_type != MocType::FrontendApp || manifest.language != "tauri_ts" {
        return None;
    }

    let cargo_manifest = moc_root.join("src-tauri").join("Cargo.toml");
    cargo_manifest.is_file().then_some(cargo_manifest)
}

fn resolve_diagnostics_root(blocks_root: &str) -> PathBuf {
    let root = Path::new(blocks_root);
    let workspace_root = root.parent().unwrap_or(root);
    workspace_root.join(".blocks").join("diagnostics")
}

fn resolve_browser_preview_root(blocks_root: &str, mocs_root: &Path) -> PathBuf {
    let blocks_root = fs::canonicalize(blocks_root).unwrap_or_else(|_| PathBuf::from(blocks_root));
    let mocs_root = fs::canonicalize(mocs_root).unwrap_or_else(|_| mocs_root.to_path_buf());
    let blocks_parent = blocks_root.parent().unwrap_or(blocks_root.as_path());

    common_path_prefix(blocks_parent, &mocs_root).unwrap_or_else(|| PathBuf::from("."))
}

fn common_path_prefix(left: &Path, right: &Path) -> Option<PathBuf> {
    let mut shared = PathBuf::new();
    let mut matched = false;

    for (left_component, right_component) in left.components().zip(right.components()) {
        if left_component != right_component {
            break;
        }

        shared.push(left_component.as_os_str());
        matched = true;
    }

    matched.then_some(shared)
}

fn render_browser_preview_lines(preview_path: &Path, workspace_root: &Path) -> Vec<String> {
    const DEFAULT_PORT: u16 = 4173;

    let preview_target = preview_path
        .strip_prefix(workspace_root)
        .unwrap_or(preview_path);
    let mut browser_path = preview_target.to_string_lossy().replace('\\', "/");
    if !browser_path.starts_with('/') {
        browser_path.insert(0, '/');
    }

    vec![
        format!(
            "browser preview: python3 -m http.server --directory {} {DEFAULT_PORT}",
            shell_quote_path(workspace_root)
        ),
        format!("browser url: http://127.0.0.1:{DEFAULT_PORT}{browser_path}"),
    ]
}

fn shell_quote_path(path: &Path) -> String {
    let rendered = path.display().to_string();
    if rendered
        .chars()
        .any(|ch| ch.is_whitespace() || ch == '\'' || ch == '"')
    {
        format!("'{}'", rendered.replace('\'', "'\\''"))
    } else {
        rendered
    }
}

fn run_real_frontend_host(cargo_manifest: &Path) -> Result<String, String> {
    let output = Command::new("cargo")
        .arg("--offline")
        .arg("run")
        .arg("--manifest-path")
        .arg(cargo_manifest)
        .arg("--")
        .arg("--headless-probe")
        .output()
        .map_err(|error| {
            format!(
                "failed to launch frontend host {}: {error}",
                cargo_manifest.display()
            )
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!(
                "frontend host failed: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ))
        } else {
            Err(stderr)
        }
    }
}

fn read_default_moc_input(moc_root: &Path) -> Result<Value, String> {
    let input_path = moc_root.join("input.example.json");
    let path = input_path.display().to_string();
    read_json_file(&path)
}

fn read_json_file(path: &str) -> Result<Value, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read input file {path}: {error}"))?;

    serde_json::from_str(&source)
        .map_err(|error| format!("failed to parse input JSON {path}: {error}"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use blocks_moc::MocManifest;
    use serde_json::Value;
    use tempfile::TempDir;

    use super::{
        resolve_frontend_host_launcher, resolve_frontend_lib_preview, resolve_frontend_preview,
        resolve_rust_backend_launcher, resolve_rust_lib_manifest, run,
    };

    fn write_block(root: &std::path::Path, dir_name: &str, id: &str, body: &str) {
        fn has_key(source: &str, key: &str) -> bool {
            source
                .lines()
                .any(|line| line.trim_start().starts_with(&format!("{key}:")))
        }

        fn ensure_key(source: &mut String, key: &str, snippet: &str) {
            if !has_key(source, key) {
                source.push_str(snippet);
            }
        }

        let mut content = body.to_string();
        ensure_key(&mut content, "version", "version: 0.1.0\n");
        ensure_key(&mut content, "status", "status: candidate\n");
        ensure_key(&mut content, "owner", "owner: blocks-core-team\n");
        ensure_key(&mut content, "purpose", "purpose: test block\n");
        ensure_key(&mut content, "scope", "scope:\n  - test scope\n");
        ensure_key(&mut content, "non_goals", "non_goals:\n  - test non-goal\n");
        ensure_key(
            &mut content,
            "inputs",
            "inputs:\n  - name: text\n    description: input\n",
        );
        ensure_key(
            &mut content,
            "input_schema",
            "input_schema:\n  text:\n    type: string\n    required: true\n",
        );
        ensure_key(
            &mut content,
            "preconditions",
            "preconditions:\n  - input exists\n",
        );
        ensure_key(
            &mut content,
            "outputs",
            "outputs:\n  - name: text\n    description: output\n",
        );
        ensure_key(
            &mut content,
            "output_schema",
            "output_schema:\n  text:\n    type: string\n    required: true\n",
        );
        ensure_key(
            &mut content,
            "postconditions",
            "postconditions:\n  - output exists\n",
        );
        ensure_key(
            &mut content,
            "dependencies",
            "dependencies:\n  runtime:\n    - std\n",
        );
        ensure_key(&mut content, "side_effects", "side_effects:\n  - none\n");
        ensure_key(&mut content, "timeouts", "timeouts:\n  default_ms: 100\n");
        ensure_key(
            &mut content,
            "resource_limits",
            "resource_limits:\n  memory_mb: 16\n",
        );
        ensure_key(
            &mut content,
            "failure_modes",
            "failure_modes:\n  - id: invalid_input\n    when: invalid input\n",
        );
        ensure_key(
            &mut content,
            "error_codes",
            "error_codes:\n  - invalid_input\n",
        );
        ensure_key(
            &mut content,
            "recovery_strategy",
            "recovery_strategy:\n  - retry\n",
        );
        ensure_key(
            &mut content,
            "verification",
            "verification:\n  automated:\n    - cargo test\n",
        );
        ensure_key(
            &mut content,
            "evaluation",
            "evaluation:\n  quality_gates:\n    - stable\n",
        );
        ensure_key(
            &mut content,
            "acceptance_criteria",
            "acceptance_criteria:\n  - works\n",
        );
        ensure_key(
            &mut content,
            "debug",
            "debug:\n  enabled_in_dev: true\n  emits_structured_logs: true\n  log_fields:\n    - execution_id\n",
        );
        ensure_key(
            &mut content,
            "observe",
            "observe:\n  metrics:\n    - execution_total\n  emits_failure_artifact: true\n  artifact_policy:\n    mode: on_failure\n",
        );
        ensure_key(
            &mut content,
            "errors",
            "errors:\n  taxonomy:\n    - id: invalid_input\n    - id: internal_error\n",
        );

        let block_dir = root.join(dir_name);
        let rust_dir = block_dir.join("rust");
        fs::create_dir_all(&rust_dir).expect("block dir should be created");
        fs::write(block_dir.join("block.yaml"), content).expect("contract should be written");
        fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
        let _ = id;
    }

    #[test]
    fn runs_demo_echo_block_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let input_path = temp_dir.path().join("input.json");
        fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

        let output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "demo.echo".to_string(),
            input_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("\"text\": \"hello\""));
    }

    #[test]
    fn validates_moc_manifest_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_path = temp_dir.path().join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#,
        )
        .expect("manifest should be written");
        let output = run(vec![
            "moc".to_string(),
            "validate".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("valid: echo-pipeline"));
        assert!(output.contains("type=backend_app"));
        assert!(output.contains("backend_mode=console"));
        assert!(output.contains("steps=1"));
    }

    #[test]
    fn validates_descriptor_only_moc_manifest() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        fs::create_dir_all(&blocks_root).expect("blocks root should be created");

        let manifest_path = temp_dir.path().join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: hello-world-console
name: Hello World Console
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    written:
      type: boolean
      required: true
uses:
  blocks:
    - core.console.write_line
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: stdout-line
    channel: stdio
    input_schema:
      text:
        type: string
        required: true
    output_schema:
      written:
        type: boolean
        required: true
verification:
  commands:
    - cargo test
acceptance_criteria:
  - prints the provided text exactly once
"#,
        )
        .expect("manifest should be written");
        let output = run(vec![
            "moc".to_string(),
            "validate".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("valid: hello-world-console"));
        assert!(output.contains("descriptor_only=true"));
    }

    #[test]
    fn runs_moc_validation_flow_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_dir = temp_dir.path().join("echo-pipeline");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
        let manifest_path = manifest_dir.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#,
        )
        .expect("manifest should be written");

        let input_path = temp_dir.path().join("input.json");
        fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

        let output = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            input_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("\"text\": \"hello\""));
    }

    #[test]
    fn runs_validation_flow_through_moc_run_when_no_launcher_exists() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_dir = temp_dir.path().join("echo-pipeline");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
        let manifest_path = manifest_dir.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#,
        )
        .expect("manifest should be written");
        fs::write(
            manifest_dir.join("input.example.json"),
            r#"{ "text": "hello" }"#,
        )
        .expect("input should be written");

        let output = run(vec![
            "moc".to_string(),
            "run".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should run flow via runtime wrapper");

        assert!(output.contains("\"text\":"));
        assert!(output.contains("trace_id:"));
    }

    #[test]
    fn resolves_rust_backend_launcher_from_manifest_entry() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let moc_root = temp_dir.path().join("hello-world-console");
        let backend_src = moc_root.join("backend").join("src");
        fs::create_dir_all(&backend_src).expect("backend src dir should be created");
        fs::write(
            moc_root.join("backend").join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("cargo manifest should be written");
        fs::write(backend_src.join("main.rs"), "fn main() {}\n")
            .expect("main.rs should be written");

        let manifest = MocManifest::from_yaml_str(
            r#"
id: hello-world-console
name: Hello World Console
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
acceptance_criteria:
  - prints hello world
"#,
        )
        .expect("manifest should parse");

        let launcher =
            resolve_rust_backend_launcher(&manifest, &moc_root).expect("launcher should resolve");

        assert_eq!(launcher, moc_root.join("backend").join("Cargo.toml"));
    }

    #[test]
    fn runs_frontend_moc_from_preview_path() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        fs::create_dir_all(&blocks_root).expect("blocks root should be created");

        let moc_root = temp_dir.path().join("counter-panel-web");
        let preview_dir = moc_root.join("preview");
        fs::create_dir_all(&preview_dir).expect("preview dir should be created");
        fs::write(
            preview_dir.join("index.html"),
            "<!doctype html>\n<title>preview</title>\n",
        )
        .expect("preview should be written");

        let manifest_path = moc_root.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: counter-panel-web
name: Counter Panel Web
type: frontend_app
language: tauri_ts
entry: src/main.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks:
    - ui.counter.mount
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: dom-ready
    channel: webview
    input_schema: {}
    output_schema:
      mounted:
        type: boolean
        required: true
verification:
  commands:
    - review src/main.ts and preview/index.html
acceptance_criteria:
  - mounts a counter into #app
"#,
        )
        .expect("manifest should be written");

        let output = run(vec![
            "moc".to_string(),
            "run".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("frontend preview:"));
        assert!(output.contains("preview/index.html"));
    }

    #[test]
    fn resolves_frontend_preview_from_conventional_path() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let moc_root = temp_dir.path().join("counter-panel-web");
        let preview_dir = moc_root.join("preview");
        fs::create_dir_all(&preview_dir).expect("preview dir should be created");
        fs::write(preview_dir.join("index.html"), "<!doctype html>\n")
            .expect("preview should be written");

        let manifest = MocManifest::from_yaml_str(
            r#"
id: counter-panel-web
name: Counter Panel Web
type: frontend_app
language: tauri_ts
entry: src/main.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks:
    - ui.counter.mount
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: dom-ready
    channel: webview
    input_schema: {}
    output_schema:
      mounted:
        type: boolean
        required: true
verification:
  commands:
    - review src/main.ts and preview/index.html
acceptance_criteria:
  - mounts a counter into #app
"#,
        )
        .expect("manifest should parse");

        let preview =
            resolve_frontend_preview(&manifest, &moc_root).expect("preview should resolve");

        assert_eq!(preview, moc_root.join("preview").join("index.html"));
    }

    #[test]
    fn resolves_frontend_host_launcher_from_src_tauri_manifest() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let moc_root = temp_dir.path().join("counter-panel-web");
        let host_dir = moc_root.join("src-tauri");
        fs::create_dir_all(&host_dir).expect("host dir should be created");
        fs::write(
            host_dir.join("Cargo.toml"),
            "[package]\nname = \"counter-panel-web-host\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("cargo manifest should be written");

        let manifest = MocManifest::from_yaml_str(
            r#"
id: counter-panel-web
name: Counter Panel Web
type: frontend_app
language: tauri_ts
entry: src/main.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks:
    - ui.counter.mount
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: dom-ready
    channel: webview
    input_schema: {}
    output_schema:
      mounted:
        type: boolean
        required: true
verification:
  commands:
    - review src/main.ts and src-tauri
acceptance_criteria:
  - mounts a counter into #app
"#,
        )
        .expect("manifest should parse");

        let launcher =
            resolve_frontend_host_launcher(&manifest, &moc_root).expect("host should resolve");

        assert_eq!(launcher, host_dir.join("Cargo.toml"));
    }

    #[test]
    fn resolves_rust_lib_manifest_from_moc_root() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let moc_root = temp_dir.path().join("hello-message-lib");
        fs::create_dir_all(&moc_root).expect("moc root should be created");
        fs::write(
            moc_root.join("Cargo.toml"),
            "[package]\nname = \"hello-message-lib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("cargo manifest should be written");

        let cargo_manifest =
            resolve_rust_lib_manifest(&moc_root).expect("rust lib manifest should resolve");

        assert_eq!(cargo_manifest, moc_root.join("Cargo.toml"));
    }

    #[test]
    fn resolves_frontend_lib_preview_from_conventional_path() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let moc_root = temp_dir.path().join("hello-panel-lib");
        let preview_dir = moc_root.join("preview");
        fs::create_dir_all(&preview_dir).expect("preview dir should be created");
        fs::write(preview_dir.join("index.html"), "<!doctype html>\n")
            .expect("preview should be written");

        let preview =
            resolve_frontend_lib_preview(&moc_root).expect("frontend lib preview should resolve");

        assert_eq!(preview, moc_root.join("preview").join("index.html"));
    }

    #[test]
    fn runs_rust_lib_dev_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        fs::create_dir_all(&blocks_root).expect("blocks root should be created");

        let moc_root = temp_dir.path().join("hello-message-lib");
        let src_dir = moc_root.join("src");
        fs::create_dir_all(&src_dir).expect("src dir should be created");
        fs::write(
            moc_root.join("Cargo.toml"),
            r#"
[package]
name = "temp-hello-message-lib"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"
"#,
        )
        .expect("cargo manifest should be written");
        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub fn hello_message() -> &'static str {
    "hello world"
}

#[cfg(test)]
mod tests {
    use super::hello_message;

    #[test]
    fn returns_expected_message() {
        assert_eq!(hello_message(), "hello world");
    }
}
"#,
        )
        .expect("lib.rs should be written");
        let manifest_path = moc_root.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: hello-message-lib
name: Hello Message Lib
type: rust_lib
language: rust
entry: src/lib.rs
public_contract:
  input_schema: {}
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: hello-message
    channel: memory
    input_schema: {}
    output_schema:
      text:
        type: string
        required: true
verification:
  commands:
    - cargo test
acceptance_criteria:
  - returns the fixed hello world message
"#,
        )
        .expect("manifest should be written");

        let output = run(vec![
            "moc".to_string(),
            "dev".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("rust lib dev ok:"));
        assert!(output.contains("Cargo.toml"));
    }

    #[test]
    fn runs_frontend_lib_dev_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        fs::create_dir_all(&blocks_root).expect("blocks root should be created");

        let moc_root = temp_dir.path().join("hello-panel-lib");
        let preview_dir = moc_root.join("preview");
        fs::create_dir_all(&preview_dir).expect("preview dir should be created");
        fs::write(
            preview_dir.join("index.html"),
            "<!doctype html>\n<title>preview</title>\n",
        )
        .expect("preview should be written");
        let src_dir = moc_root.join("src");
        fs::create_dir_all(&src_dir).expect("src dir should be created");
        fs::write(
            src_dir.join("index.ts"),
            "export function mountHelloPanel() {}\n",
        )
        .expect("entry should be written");
        let manifest_path = moc_root.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: hello-panel-lib
name: Hello Panel Lib
type: frontend_lib
language: tauri_ts
entry: src/index.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks:
    - ui.dom.mount_text
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: mount-hello-panel
    channel: webview
    input_schema: {}
    output_schema:
      mounted:
        type: boolean
        required: true
verification:
  commands:
    - review src/index.ts and preview/index.html
acceptance_criteria:
  - exports a reusable frontend function that mounts the hello text
"#,
        )
        .expect("manifest should be written");

        let output = run(vec![
            "moc".to_string(),
            "dev".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("frontend lib preview:"));
        assert!(output.contains("preview/index.html"));
        assert!(output.contains("browser preview: python3 -m http.server --directory"));
        assert!(
            output
                .contains("browser url: http://127.0.0.1:4173/hello-panel-lib/preview/index.html")
        );
    }

    #[test]
    fn reports_helpful_missing_bind_error_from_moc_verify() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_path = temp_dir.path().join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: missing-bind
name: Missing Bind
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds: []
acceptance_criteria:
  - reports missing bind
"#,
        )
        .expect("manifest should be written");

        let error = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect_err("command should fail");

        assert!(error.contains("moc verify failed"));
        assert!(error.contains("missing bind for required field echo.text"));
        assert!(error.contains("verification.flows[plan] step `echo`"));
        assert!(error.contains("Add a bind entry under that flow"));
    }

    #[test]
    fn reports_helpful_type_mismatch_error_from_moc_verify() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_path = temp_dir.path().join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: type-mismatch
name: Type Mismatch
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: number
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - reports type mismatch
"#,
        )
        .expect("manifest should be written");

        let error = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect_err("command should fail");

        assert!(error.contains("moc verify failed"));
        assert!(
            error.contains(
                "type mismatch in flow plan step echo bind #1 from input.text to echo.text"
            )
        );
        assert!(error.contains("verification.flows[plan] step `echo` bind #1"));
        assert!(error.contains("wrong source type"));
    }

    #[test]
    fn reports_helpful_missing_input_reference_error_from_moc_verify() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_dir = temp_dir.path().join("echo-pipeline");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
        let manifest_path = manifest_dir.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - reports missing input field
"#,
        )
        .expect("manifest should be written");
        let input_path = manifest_dir.join("input.json");
        fs::write(&input_path, r#"{ "other": "hello" }"#).expect("input should be written");

        let error = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            input_path.display().to_string(),
        ])
        .expect_err("command should fail");

        assert!(error.contains("moc verify failed"));
        assert!(error.contains("invalid reference in flow plan step echo bind #1"));
        assert!(error.contains("verification.flows[plan] step `echo` bind #1"));
        assert!(error.contains("referenced field must exist in the verify input"));
    }

    #[test]
    fn reports_frontend_app_human_dev_paths_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        fs::create_dir_all(&blocks_root).expect("blocks root should be created");

        let moc_root = temp_dir.path().join("counter-panel-web");
        let preview_dir = moc_root.join("preview");
        fs::create_dir_all(&preview_dir).expect("preview dir should be created");
        fs::write(
            preview_dir.join("index.html"),
            "<!doctype html>\n<title>preview</title>\n",
        )
        .expect("preview should be written");
        let host_dir = moc_root.join("src-tauri");
        fs::create_dir_all(&host_dir).expect("host dir should be created");
        fs::write(
            host_dir.join("Cargo.toml"),
            "[package]\nname = \"counter-panel-web-host\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("cargo manifest should be written");

        let manifest_path = moc_root.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: counter-panel-web
name: Counter Panel Web
type: frontend_app
language: tauri_ts
entry: src/main.ts
public_contract:
  input_schema: {}
  output_schema:
    mounted:
      type: boolean
      required: true
uses:
  blocks:
    - ui.counter.mount
  internal_blocks: []
depends_on_mocs: []
protocols:
  - name: dom-ready
    channel: webview
    input_schema: {}
    output_schema:
      mounted:
        type: boolean
        required: true
verification:
  commands:
    - cargo --offline run --manifest-path src-tauri/Cargo.toml -- --headless-probe
acceptance_criteria:
  - renders a counter card into the #app element
"#,
        )
        .expect("manifest should be written");

        let output = run(vec![
            "moc".to_string(),
            "dev".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("frontend app dev: counter-panel-web"));
        assert!(output.contains("web preview:"));
        assert!(output.contains("browser preview: python3 -m http.server --directory"));
        assert!(
            output.contains(
                "browser url: http://127.0.0.1:4173/counter-panel-web/preview/index.html"
            )
        );
        assert!(output.contains("linux app: cargo run --manifest-path"));
        assert!(output.contains("linux app (headless probe):"));
    }

    #[test]
    fn shows_resolved_implementation_path() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
"#,
        );

        let output = run(vec![
            "show".to_string(),
            blocks_root.display().to_string(),
            "demo.echo".to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("implementation:"));
        assert!(output.contains("rust/lib.rs"));
    }

    #[test]
    fn supports_block_diagnose_json_output() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
errors:
  taxonomy:
    - id: invalid_input
    - id: internal_error
"#,
        );

        let input_path = temp_dir.path().join("input.json");
        fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");
        run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "demo.echo".to_string(),
            input_path.display().to_string(),
        ])
        .expect("block run should produce diagnostics");

        let output = run(vec![
            "block".to_string(),
            "diagnose".to_string(),
            blocks_root.display().to_string(),
            "demo.echo".to_string(),
            "--json".to_string(),
        ])
        .expect("diagnose command should succeed");

        let payload: Value =
            serde_json::from_str(&output).expect("diagnose output should be valid json");
        assert_eq!(payload["block_id"], "demo.echo");
        assert!(payload["execution_id"].is_string());
    }

    #[test]
    fn supports_moc_diagnose_json_trace_chain() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_dir = temp_dir.path().join("echo-pipeline");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
        let manifest_path = manifest_dir.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: first
          block: demo.echo
        - id: second
          block: demo.echo
      binds:
        - from: input.text
          to: first.text
        - from: first.text
          to: second.text
acceptance_criteria:
  - echoes the provided text twice
"#,
        )
        .expect("manifest should be written");
        let input_path = manifest_dir.join("input.example.json");
        fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

        let verify_output = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            input_path.display().to_string(),
        ])
        .expect("moc verify should succeed");
        let trace_id = verify_output
            .lines()
            .find_map(|line| line.strip_prefix("trace_id: "))
            .expect("trace_id should be present in verify output")
            .to_string();

        let output = run(vec![
            "moc".to_string(),
            "diagnose".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            "--trace-id".to_string(),
            trace_id.clone(),
            "--json".to_string(),
        ])
        .expect("moc diagnose command should succeed");

        let payload: Value =
            serde_json::from_str(&output).expect("moc diagnose output should be valid json");
        assert_eq!(payload["trace_id"].as_str(), Some(trace_id.as_str()));
        let entries = payload["events"]
            .as_array()
            .expect("events should be an array");
        assert!(
            entries.len() >= 2,
            "diagnose trace should include at least two block executions"
        );
        for entry in entries {
            assert_eq!(
                entry["trace_id"].as_str(),
                Some(trace_id.as_str()),
                "all executions in the chain must share the same trace_id"
            );
        }
    }

    #[test]
    fn selects_latest_moc_trace_by_moc_id_before_block_usage_fallback() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let write_moc_manifest = |dir_name: &str, moc_id: &str| -> std::path::PathBuf {
            let manifest_dir = temp_dir.path().join(dir_name);
            fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
            let manifest_path = manifest_dir.join("moc.yaml");
            fs::write(
                &manifest_path,
                format!(
                    r#"
id: {moc_id}
name: {moc_id}
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#
                ),
            )
            .expect("manifest should be written");
            fs::write(
                manifest_dir.join("input.example.json"),
                format!(r#"{{ "text": "{moc_id}" }}"#),
            )
            .expect("input should be written");
            manifest_path
        };

        let primary_manifest_path = write_moc_manifest("primary-moc", "primary-moc");
        let secondary_manifest_path = write_moc_manifest("secondary-moc", "secondary-moc");

        let primary_verify_output = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            primary_manifest_path.display().to_string(),
        ])
        .expect("primary moc verify should succeed");
        let primary_trace_id = primary_verify_output
            .lines()
            .find_map(|line| line.strip_prefix("trace_id: "))
            .expect("primary trace id should exist")
            .to_string();

        let secondary_verify_output = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            secondary_manifest_path.display().to_string(),
        ])
        .expect("secondary moc verify should succeed");
        let secondary_trace_id = secondary_verify_output
            .lines()
            .find_map(|line| line.strip_prefix("trace_id: "))
            .expect("secondary trace id should exist")
            .to_string();
        assert_ne!(
            primary_trace_id, secondary_trace_id,
            "two verifies should produce different trace ids"
        );

        let output = run(vec![
            "moc".to_string(),
            "diagnose".to_string(),
            blocks_root.display().to_string(),
            primary_manifest_path.display().to_string(),
            "--json".to_string(),
        ])
        .expect("moc diagnose command should succeed");
        let payload: Value =
            serde_json::from_str(&output).expect("moc diagnose output should be valid json");
        assert_eq!(
            payload["trace_id"].as_str(),
            Some(primary_trace_id.as_str()),
            "latest trace selection must prefer manifest moc_id"
        );
    }

    #[test]
    fn includes_moc_id_in_moc_diagnose_events() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_dir = temp_dir.path().join("echo-pipeline");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
        let manifest_path = manifest_dir.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: echo
          block: demo.echo
      binds:
        - from: input.text
          to: echo.text
acceptance_criteria:
  - echoes the provided text
"#,
        )
        .expect("manifest should be written");
        fs::write(
            manifest_dir.join("input.example.json"),
            r#"{ "text": "hello" }"#,
        )
        .expect("input should be written");

        let verify_output = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("moc verify should succeed");
        let trace_id = verify_output
            .lines()
            .find_map(|line| line.strip_prefix("trace_id: "))
            .expect("trace id should be present")
            .to_string();

        let output = run(vec![
            "moc".to_string(),
            "diagnose".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            "--trace-id".to_string(),
            trace_id,
            "--json".to_string(),
        ])
        .expect("moc diagnose should succeed");

        let payload: Value =
            serde_json::from_str(&output).expect("moc diagnose output should be valid json");
        let entries = payload["events"]
            .as_array()
            .expect("events should be an array");
        assert!(!entries.is_empty(), "diagnose should return runtime events");
        for entry in entries {
            assert_eq!(
                entry["moc_id"].as_str(),
                Some("echo-pipeline"),
                "runtime events should carry moc_id for stable ownership"
            );
        }
    }

    #[test]
    fn flow_based_moc_run_and_verify_produce_correlatable_diagnostics_fields() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_dir = temp_dir.path().join("echo-pipeline");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
        let manifest_path = manifest_dir.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: first
          block: demo.echo
        - id: second
          block: demo.echo
      binds:
        - from: input.text
          to: first.text
        - from: first.text
          to: second.text
acceptance_criteria:
  - echoes the provided text twice
"#,
        )
        .expect("manifest should be written");
        let input_path = manifest_dir.join("input.example.json");
        fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

        let run_output = run(vec![
            "moc".to_string(),
            "run".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            input_path.display().to_string(),
        ])
        .expect("flow-based moc run should succeed");
        let run_trace_id = run_output
            .lines()
            .find_map(|line| line.strip_prefix("trace_id: "))
            .expect("moc run should return trace_id")
            .to_string();

        let verify_output = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            input_path.display().to_string(),
        ])
        .expect("moc verify should succeed");
        let verify_trace_id = verify_output
            .lines()
            .find_map(|line| line.strip_prefix("trace_id: "))
            .expect("moc verify should return trace_id")
            .to_string();

        assert!(run_output.contains("\"text\": \"hello\""));
        assert!(verify_output.contains("\"text\": \"hello\""));

        for trace_id in [&run_trace_id, &verify_trace_id] {
            let diagnose_output = run(vec![
                "moc".to_string(),
                "diagnose".to_string(),
                blocks_root.display().to_string(),
                manifest_path.display().to_string(),
                "--trace-id".to_string(),
                trace_id.to_string(),
                "--json".to_string(),
            ])
            .expect("moc diagnose should succeed");

            let payload: Value = serde_json::from_str(&diagnose_output)
                .expect("moc diagnose output should be valid json");
            let entries = payload["events"]
                .as_array()
                .expect("events should be an array");
            assert!(
                !entries.is_empty(),
                "trace should include diagnostics events"
            );
            for entry in entries {
                assert_eq!(entry["trace_id"].as_str(), Some(trace_id.as_str()));
                assert_eq!(entry["moc_id"].as_str(), Some("echo-pipeline"));
                assert!(entry["execution_id"].as_str().is_some());
            }
        }
    }

    #[test]
    fn redacts_sensitive_values_in_moc_diagnose_artifacts() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
version: 1.0.0
status: active
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
  token:
    type: string
    required: true
"#,
        );
        let manifest_dir = temp_dir.path().join("echo-pipeline");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should be created");
        let manifest_path = manifest_dir.join("moc.yaml");
        fs::write(
            &manifest_path,
            r#"
id: echo-pipeline
name: Echo Pipeline
type: backend_app
backend_mode: console
language: rust
entry: backend/src/main.rs
public_contract:
  input_schema:
    text:
      type: string
      required: true
  output_schema:
    text:
      type: string
      required: true
uses:
  blocks:
    - demo.echo
  internal_blocks: []
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
  entry_flow: plan
  flows:
    - id: plan
      steps:
        - id: first
          block: demo.echo
      binds:
        - from: input.text
          to: first.text
acceptance_criteria:
  - echoes the provided text
"#,
        )
        .expect("manifest should be written");
        let input_path = manifest_dir.join("input.example.json");
        fs::write(&input_path, r#"{ "text": "Bearer super-secret-token" }"#)
            .expect("input should be written");

        let _verify_error = run(vec![
            "moc".to_string(),
            "verify".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            input_path.display().to_string(),
        ])
        .expect_err("moc verify should fail and emit diagnostics");

        let output = run(vec![
            "moc".to_string(),
            "diagnose".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            "--json".to_string(),
        ])
        .expect("moc diagnose command should succeed");

        let payload: Value =
            serde_json::from_str(&output).expect("moc diagnose output should be valid json");
        let artifacts = payload["artifacts"]
            .as_array()
            .expect("artifacts should be an array");
        let artifact = artifacts
            .iter()
            .find_map(|item| item.as_object())
            .expect("at least one artifact should be present");
        assert_eq!(
            artifact["input_snapshot"]["text"].as_str(),
            Some("***REDACTED***")
        );
    }
}
