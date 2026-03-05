use blocks_moc::{MocComposer, MocManifest, MocType};
use blocks_registry::Registry;
use blocks_runtime::{DiagnosticEvent, read_diagnostic_artifact, read_diagnostic_events};
use serde_json::json;

use crate::app::{
    execute_moc_flow_runtime_wrapper, load_moc_manifest, render_moc_verify_error,
    resolve_browser_preview_root, resolve_diagnostics_root, resolve_frontend_host_launcher,
    resolve_frontend_lib_preview, resolve_frontend_preview, resolve_rust_backend_launcher,
    resolve_rust_lib_manifest, run_real_frontend_host, run_real_rust_backend, run_rust_lib_dev,
    validate_moc_manifest,
};
use crate::render::{render_browser_preview_lines, render_moc_diagnose_human};

pub fn validate_command(root: &str, manifest_path: &str) -> Result<String, String> {
    let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    validate_moc_manifest(&manifest, moc_root.as_path(), mocs_root.as_path())?;
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

    Ok(format!(
        "valid: {} ({}) {}",
        manifest.id,
        manifest_path,
        details.join(" ")
    ))
}

pub fn run_command(
    root: &str,
    manifest_path: &str,
    input_path: Option<&str>,
) -> Result<String, String> {
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    validate_moc_manifest(&manifest, moc_root.as_path(), mocs_root.as_path())?;

    if let Some(cargo_manifest) = resolve_rust_backend_launcher(&manifest, moc_root.as_path()) {
        return run_real_rust_backend(&cargo_manifest, input_path);
    }

    if let Some(cargo_manifest) = resolve_frontend_host_launcher(&manifest, moc_root.as_path()) {
        return run_real_frontend_host(&cargo_manifest);
    }

    if let Some(preview_path) = resolve_frontend_preview(&manifest, moc_root.as_path()) {
        return Ok(format!("frontend preview: {}", preview_path.display()));
    }

    if manifest.has_validation_flow() {
        let (output, trace_id) =
            execute_moc_flow_runtime_wrapper(root, &manifest, moc_root.as_path(), input_path)
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

pub fn verify_command(
    root: &str,
    manifest_path: &str,
    input_path: Option<&str>,
) -> Result<String, String> {
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    validate_moc_manifest(&manifest, moc_root.as_path(), mocs_root.as_path())?;
    let (output, trace_id) =
        execute_moc_flow_runtime_wrapper(root, &manifest, moc_root.as_path(), input_path)
            .map_err(|error| render_moc_verify_error(manifest_path, &error))?;
    let rendered = serde_json::to_string_pretty(&output)
        .map_err(|error| format!("failed to render output JSON: {error}"))?;
    Ok(format!("{rendered}\ntrace_id: {trace_id}"))
}

pub fn dev_command(blocks_root: &str, manifest_path: &str) -> Result<String, String> {
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    let workspace_root = resolve_browser_preview_root(blocks_root, mocs_root.as_path());
    validate_moc_manifest(&manifest, moc_root.as_path(), mocs_root.as_path())?;

    match manifest.moc_type {
        MocType::RustLib => {
            let cargo_manifest =
                resolve_rust_lib_manifest(moc_root.as_path()).ok_or_else(|| {
                    format!("rust_lib dev requires Cargo.toml in {}", moc_root.display())
                })?;
            run_rust_lib_dev(&cargo_manifest)
        }
        MocType::FrontendLib => {
            if let Some(preview_path) = resolve_frontend_lib_preview(moc_root.as_path()) {
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

            if let Some(preview_path) = resolve_frontend_preview(&manifest, moc_root.as_path()) {
                lines.push(format!("web preview: {}", preview_path.display()));
                lines.extend(render_browser_preview_lines(&preview_path, &workspace_root));
            }

            if let Some(cargo_manifest) =
                resolve_frontend_host_launcher(&manifest, moc_root.as_path())
            {
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

#[derive(Default)]
struct MocDiagnoseOptions {
    trace_id: Option<String>,
    json: bool,
}

pub fn diagnose_command(
    blocks_root: &str,
    manifest_path: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_diagnose_options(args)?;
    let (manifest, moc_root, mocs_root) = load_moc_manifest(manifest_path)?;
    validate_moc_manifest(&manifest, moc_root.as_path(), mocs_root.as_path())?;

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

fn parse_diagnose_options(args: &[String]) -> Result<MocDiagnoseOptions, String> {
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
