use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use blocks_moc::{ExecutionPlan, MocComposer, MocError, MocManifest, MocType};
use blocks_registry::Registry;
use blocks_runner_catalog::default_block_runner;
use blocks_runtime::{BlockRunner, ExecutionContext, Runtime, generate_trace_id};
use serde_json::Value;

pub fn load_moc_manifest(manifest_path: &str) -> Result<(MocManifest, PathBuf, PathBuf), String> {
    let manifest_source = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read moc manifest {manifest_path}: {error}"))?;
    let manifest = MocManifest::from_yaml_str(&manifest_source)
        .map_err(|error| format!("failed to load moc manifest {manifest_path}: {error}"))?;
    let moc_root = Path::new(manifest_path)
        .parent()
        .ok_or_else(|| format!("invalid moc manifest path: {manifest_path}"))?
        .to_path_buf();
    let mocs_root = moc_root
        .parent()
        .unwrap_or(moc_root.as_path())
        .to_path_buf();

    Ok((manifest, moc_root, mocs_root))
}

pub fn validate_moc_manifest(
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

pub fn execute_moc_flow_runtime_wrapper(
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

pub fn execute_validation_plan(
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

pub fn render_moc_verify_error(manifest_path: &str, error: &MocError) -> String {
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

pub fn resolve_rust_backend_launcher(manifest: &MocManifest, moc_root: &Path) -> Option<PathBuf> {
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

pub fn resolve_rust_lib_manifest(moc_root: &Path) -> Option<PathBuf> {
    let cargo_manifest = moc_root.join("Cargo.toml");
    cargo_manifest.is_file().then_some(cargo_manifest)
}

pub fn run_real_rust_backend(
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

pub fn run_rust_lib_dev(cargo_manifest: &Path) -> Result<String, String> {
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

pub fn resolve_frontend_preview(manifest: &MocManifest, moc_root: &Path) -> Option<PathBuf> {
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

pub fn resolve_frontend_lib_preview(moc_root: &Path) -> Option<PathBuf> {
    let preview_index = moc_root.join("preview").join("index.html");
    if preview_index.is_file() {
        return Some(preview_index);
    }

    let preview_html = moc_root.join("preview.html");
    preview_html.is_file().then_some(preview_html)
}

pub fn resolve_frontend_host_launcher(manifest: &MocManifest, moc_root: &Path) -> Option<PathBuf> {
    if manifest.moc_type != MocType::FrontendApp || manifest.language != "tauri_ts" {
        return None;
    }

    let cargo_manifest = moc_root.join("src-tauri").join("Cargo.toml");
    cargo_manifest.is_file().then_some(cargo_manifest)
}

pub fn resolve_diagnostics_root(blocks_root: &str) -> PathBuf {
    let root = Path::new(blocks_root);
    let workspace_root = root.parent().unwrap_or(root);
    workspace_root.join(".blocks").join("diagnostics")
}

pub fn resolve_browser_preview_root(blocks_root: &str, mocs_root: &Path) -> PathBuf {
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

pub fn run_real_frontend_host(cargo_manifest: &Path) -> Result<String, String> {
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

pub fn read_default_moc_input(moc_root: &Path) -> Result<Value, String> {
    let input_path = moc_root.join("input.example.json");
    let path = input_path.display().to_string();
    read_json_file(&path)
}

pub fn read_json_file(path: &str) -> Result<Value, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read input file {path}: {error}"))?;

    serde_json::from_str(&source)
        .map_err(|error| format!("failed to parse input JSON {path}: {error}"))
}
