use std::path::{Path, PathBuf};

use blocks_moc::{BackendMode, MocComposer, MocManifest, MocType};
use blocks_registry::Registry;
use blocks_runtime::{DiagnosticEvent, read_diagnostic_artifact, read_diagnostic_events};
use serde::Serialize;
use serde_json::json;

use crate::app::toolchain::{
    build_moc_manifest_template, ensure_directory, ensure_new_directory, moc_backend_cargo_toml,
    moc_backend_main_template, moc_frontend_entry_template, moc_preview_html_template,
    moc_readme_template, moc_rust_lib_cargo_toml, moc_rust_lib_template, normalize_yaml,
    parse_backend_mode, parse_moc_type, read_text_file, resolve_descriptor_path,
    validate_moc_scaffold_shape, write_text_file,
};
use crate::app::{
    execute_moc_flow_runtime_wrapper, load_moc_manifest, render_moc_verify_error,
    resolve_browser_preview_root, resolve_diagnostics_root, resolve_frontend_host_launcher,
    resolve_frontend_lib_preview, resolve_frontend_preview, resolve_rust_backend_launcher,
    resolve_rust_lib_manifest, run_real_frontend_host, run_real_rust_backend, run_rust_lib_dev,
    validate_moc_manifest,
};
use crate::render::{render_browser_preview_lines, render_moc_diagnose_human};

#[derive(Default)]
struct MocInitOptions {
    moc_type: Option<MocType>,
    language: Option<String>,
    backend_mode: Option<BackendMode>,
}

#[derive(Default)]
struct MocCheckOptions {
    json: bool,
}

#[derive(Default)]
struct MocDoctorOptions {
    json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct MocDoctorLauncher {
    status: String,
    kind: String,
    path: Option<String>,
    preview_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct MocDoctorDiagnostic {
    trace_id: String,
    events: usize,
    failures: usize,
    last_error_id: Option<String>,
    artifacts: usize,
}

#[derive(Debug, Clone, Serialize)]
struct MocDoctorProtocolHealth {
    status: String,
    summary: String,
}

#[derive(Debug, Clone, Serialize)]
struct MocDoctorReport {
    target_kind: String,
    status: String,
    moc_id: String,
    path: String,
    check_status: String,
    descriptor_only: bool,
    warnings: Vec<String>,
    errors: Vec<String>,
    launcher: MocDoctorLauncher,
    protocol_health: MocDoctorProtocolHealth,
    latest_diagnostic: Option<MocDoctorDiagnostic>,
    recommendations: Vec<String>,
}

pub fn init_command(mocs_root: &str, moc_id: &str, args: &[String]) -> Result<String, String> {
    let options = parse_init_options(args)?;
    let moc_type = options
        .moc_type
        .ok_or_else(|| "moc init requires --type".to_string())?;
    let language = options
        .language
        .ok_or_else(|| "moc init requires --language".to_string())?;
    validate_moc_scaffold_shape(moc_type, &language, options.backend_mode)?;

    let moc_root = Path::new(mocs_root).join(moc_id);
    ensure_new_directory(&moc_root)?;
    ensure_directory(&moc_root.join("tests"))?;
    ensure_directory(&moc_root.join("examples"))?;

    let entry = default_moc_entry(moc_type, &language)?;
    let manifest =
        build_moc_manifest_template(moc_id, moc_type, &language, entry, options.backend_mode);
    let manifest_yaml = normalize_yaml(&manifest, "moc manifest")?;
    let manifest_path = moc_root.join("moc.yaml");
    write_text_file(&manifest_path, &manifest_yaml)?;
    write_text_file(
        &moc_root.join("README.md"),
        &moc_readme_template(moc_id, moc_type, &language),
    )?;
    scaffold_moc_layout(&moc_root, moc_id, moc_type, &language)?;

    Ok(format!(
        "scaffolded moc: {}\nmanifest: {}",
        moc_root.display(),
        manifest_path.display()
    ))
}

pub fn init_target_command(target: &str, args: &[String]) -> Result<String, String> {
    let (mocs_root, moc_id) = split_init_target(target)?;
    init_command(&mocs_root, &moc_id, args)
}

pub fn fmt_command(path_arg: &str) -> Result<String, String> {
    let manifest_path = resolve_moc_manifest_path(path_arg);
    let source = read_text_file(&manifest_path, "moc manifest")?;
    let manifest = MocManifest::from_yaml_str(&source).map_err(|error| {
        format!(
            "failed to format moc manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    let rendered = normalize_yaml(&manifest, "moc manifest")?;
    write_text_file(&manifest_path, &rendered)?;
    Ok(format!(
        "formatted moc manifest: {}",
        manifest_path.display()
    ))
}

pub fn check_command(root: &str, path_arg: &str, args: &[String]) -> Result<String, String> {
    let options = parse_check_options(args)?;
    let manifest_path = resolve_moc_manifest_path(path_arg);
    let source = read_text_file(&manifest_path, "moc manifest")?;
    let manifest = match MocManifest::from_yaml_str(&source) {
        Ok(manifest) => manifest,
        Err(error) => {
            return render_check_failure(
                options.json,
                &manifest_path,
                None,
                &[format!("failed to load moc manifest: {error}")],
                &[],
            );
        }
    };

    let moc_root = manifest_path
        .parent()
        .ok_or_else(|| format!("invalid moc manifest path: {}", manifest_path.display()))?;
    let mocs_root = moc_root.parent().unwrap_or(moc_root);
    let registry = match Registry::load_from_root(root) {
        Ok(registry) => registry,
        Err(error) => {
            return render_check_failure(
                options.json,
                &manifest_path,
                Some(&manifest.id),
                &[error.to_string()],
                &[],
            );
        }
    };

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if let Err(error) = validate_moc_manifest(&manifest, moc_root, mocs_root) {
        errors.push(error);
    }

    for block_id in &manifest.uses.blocks {
        if registry.get(block_id).is_none() {
            errors.push(format!("unknown block declared in uses.blocks: {block_id}"));
        }
    }

    if manifest.has_validation_flow() {
        if let Err(error) = MocComposer::new().plan(&manifest, &registry) {
            errors.push(error.to_string());
        }
    }

    let entry_path = moc_root.join(&manifest.entry);
    if !entry_path.is_file() {
        errors.push(format!("missing moc entry path: {}", entry_path.display()));
    }

    match manifest.moc_type {
        MocType::BackendApp if manifest.language == "rust" => {
            let cargo_manifest = moc_root.join("backend").join("Cargo.toml");
            if !cargo_manifest.is_file() {
                errors.push(format!(
                    "missing backend cargo manifest: {}",
                    cargo_manifest.display()
                ));
            }
        }
        MocType::RustLib if manifest.language == "rust" => {
            let cargo_manifest = moc_root.join("Cargo.toml");
            if !cargo_manifest.is_file() {
                errors.push(format!(
                    "missing rust_lib cargo manifest: {}",
                    cargo_manifest.display()
                ));
            }
        }
        MocType::FrontendLib | MocType::FrontendApp => {
            let preview_path = moc_root.join("preview").join("index.html");
            if !preview_path.is_file() {
                warnings.push(format!(
                    "preview helper is missing: {}",
                    preview_path.display()
                ));
            }
        }
        _ => {}
    }

    let status = if errors.is_empty() {
        if warnings.is_empty() { "ok" } else { "warn" }
    } else {
        "error"
    };
    let descriptor_only = !manifest.has_validation_flow();

    if options.json {
        let payload = json!({
            "status": status,
            "kind": "moc",
            "path": manifest_path.display().to_string(),
            "moc_id": manifest.id,
            "moc_type": manifest.moc_type.to_string(),
            "language": manifest.language,
            "entry": manifest.entry,
            "backend_mode": manifest.backend_mode.map(|mode| mode.to_string()),
            "descriptor_only": descriptor_only,
            "warnings": warnings,
            "errors": errors,
        });
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("failed to render moc check JSON: {error}"))?;
        return if status == "error" {
            Err(rendered)
        } else {
            Ok(rendered)
        };
    }

    let mut lines = vec![
        format!("moc check: {status}"),
        format!("id: {}", manifest.id),
        format!("path: {}", manifest_path.display()),
        format!("type: {}", manifest.moc_type),
        format!("language: {}", manifest.language),
        format!("entry: {}", manifest.entry),
        format!("descriptor_only: {descriptor_only}"),
    ];
    if let Some(mode) = manifest.backend_mode {
        lines.push(format!("backend_mode: {mode}"));
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

pub fn doctor_command(root: &str, path_arg: &str, args: &[String]) -> Result<String, String> {
    let options = parse_doctor_options(args)?;
    let manifest_path = resolve_moc_manifest_path(path_arg);
    let manifest_path_string = manifest_path.display().to_string();
    let payload = decode_doctor_payload(check_command(
        root,
        &manifest_path_string,
        &[String::from("--json")],
    ))?;
    let (manifest, moc_root, mocs_root) = load_moc_manifest(&manifest_path_string)?;
    let launcher = detect_launcher(&manifest, moc_root.as_path());
    let protocol_health = match manifest.validate_dependencies(mocs_root.as_path()) {
        Ok(()) => MocDoctorProtocolHealth {
            status: if manifest.depends_on_mocs.is_empty() {
                "not_applicable".to_string()
            } else {
                "ok".to_string()
            },
            summary: if manifest.depends_on_mocs.is_empty() {
                "no dependent mocs declared".to_string()
            } else {
                format!(
                    "validated {} dependent protocol bindings",
                    manifest.depends_on_mocs.len()
                )
            },
        },
        Err(error) => MocDoctorProtocolHealth {
            status: "error".to_string(),
            summary: error.to_string(),
        },
    };
    let warnings = payload
        .get("warnings")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let errors = payload
        .get("errors")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let latest_diagnostic = latest_moc_diagnostic(root, &manifest.id)?;
    let recommendations = build_moc_doctor_recommendations(
        &manifest,
        &warnings,
        &errors,
        &launcher,
        &protocol_health,
        &latest_diagnostic,
    );
    let status = if !errors.is_empty() || protocol_health.status == "error" {
        "error"
    } else if !warnings.is_empty() || !recommendations.is_empty() {
        "warn"
    } else {
        "ok"
    };
    let report = MocDoctorReport {
        target_kind: "moc".to_string(),
        status: status.to_string(),
        moc_id: manifest.id.clone(),
        path: manifest_path_string,
        check_status: payload
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_string(),
        descriptor_only: !manifest.has_validation_flow(),
        warnings,
        errors,
        launcher,
        protocol_health,
        latest_diagnostic,
        recommendations,
    };
    render_moc_doctor_report(report, options.json)
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

fn parse_doctor_options(args: &[String]) -> Result<MocDoctorOptions, String> {
    let mut options = MocDoctorOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for moc doctor: {other}")),
        }
    }
    Ok(options)
}

fn resolve_moc_manifest_path(path_arg: &str) -> PathBuf {
    resolve_descriptor_path(path_arg, "moc.yaml")
}

fn split_init_target(target: &str) -> Result<(String, String), String> {
    let path = Path::new(target);
    let moc_id = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("invalid moc init target: {target}"))?;
    let mocs_root = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    Ok((mocs_root.display().to_string(), moc_id.to_string()))
}

fn parse_init_options(args: &[String]) -> Result<MocInitOptions, String> {
    let mut options = MocInitOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--type" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--type requires a value".to_string())?;
                options.moc_type = Some(parse_moc_type(value)?);
                index += 2;
            }
            "--language" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--language requires a value".to_string())?;
                options.language = Some(value.clone());
                index += 2;
            }
            "--backend-mode" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--backend-mode requires a value".to_string())?;
                options.backend_mode = Some(parse_backend_mode(value)?);
                index += 2;
            }
            other => return Err(format!("unknown option for moc init: {other}")),
        }
    }
    Ok(options)
}

fn parse_check_options(args: &[String]) -> Result<MocCheckOptions, String> {
    let mut options = MocCheckOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for moc check: {other}")),
        }
    }
    Ok(options)
}

fn default_moc_entry(moc_type: MocType, language: &str) -> Result<&'static str, String> {
    match (moc_type, language) {
        (MocType::BackendApp, "rust") => Ok("backend/src/main.rs"),
        (MocType::RustLib, "rust") => Ok("src/lib.rs"),
        (MocType::FrontendLib | MocType::FrontendApp, "tauri_ts") => Ok("src/main.ts"),
        _ => Err(format!(
            "unsupported scaffold combination for type={} language={language}",
            moc_type
        )),
    }
}

fn scaffold_moc_layout(
    moc_root: &Path,
    moc_id: &str,
    moc_type: MocType,
    language: &str,
) -> Result<(), String> {
    match (moc_type, language) {
        (MocType::BackendApp, "rust") => {
            let backend_root = moc_root.join("backend");
            let source_root = backend_root.join("src");
            ensure_directory(&source_root)?;
            write_text_file(
                &source_root.join("main.rs"),
                &moc_backend_main_template(moc_id),
            )?;
            write_text_file(
                &backend_root.join("Cargo.toml"),
                &moc_backend_cargo_toml(moc_id),
            )?;
            write_text_file(&moc_root.join("input.example.json"), "{\n}\n")?;
        }
        (MocType::RustLib, "rust") => {
            let source_root = moc_root.join("src");
            ensure_directory(&source_root)?;
            write_text_file(&source_root.join("lib.rs"), moc_rust_lib_template())?;
            write_text_file(
                &moc_root.join("Cargo.toml"),
                &moc_rust_lib_cargo_toml(moc_id),
            )?;
        }
        (MocType::FrontendLib | MocType::FrontendApp, "tauri_ts") => {
            let source_root = moc_root.join("src");
            let preview_root = moc_root.join("preview");
            ensure_directory(&source_root)?;
            ensure_directory(&preview_root)?;
            write_text_file(
                &source_root.join("main.ts"),
                &moc_frontend_entry_template(moc_id),
            )?;
            write_text_file(
                &preview_root.join("index.html"),
                &moc_preview_html_template(moc_id),
            )?;
        }
        _ => {
            return Err(format!(
                "unsupported scaffold combination for type={} language={language}",
                moc_type
            ));
        }
    }

    Ok(())
}

fn render_check_failure(
    json_output: bool,
    manifest_path: &Path,
    moc_id: Option<&str>,
    errors: &[String],
    warnings: &[String],
) -> Result<String, String> {
    if json_output {
        let payload = json!({
            "status": "error",
            "kind": "moc",
            "path": manifest_path.display().to_string(),
            "moc_id": moc_id,
            "warnings": warnings,
            "errors": errors,
        });
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("failed to render moc check JSON: {error}"))?;
        Err(rendered)
    } else {
        let mut lines = vec![
            "moc check: error".to_string(),
            format!("path: {}", manifest_path.display()),
        ];
        if let Some(moc_id) = moc_id {
            lines.push(format!("id: {moc_id}"));
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
        .map_err(|error| format!("failed to decode moc doctor JSON payload: {error}"))
}

fn detect_launcher(manifest: &MocManifest, moc_root: &Path) -> MocDoctorLauncher {
    if let Some(cargo_manifest) = resolve_rust_backend_launcher(manifest, moc_root) {
        return MocDoctorLauncher {
            status: "ok".to_string(),
            kind: "rust_backend".to_string(),
            path: Some(cargo_manifest.display().to_string()),
            preview_path: None,
        };
    }
    if let Some(cargo_manifest) = resolve_rust_lib_manifest(moc_root) {
        return MocDoctorLauncher {
            status: "ok".to_string(),
            kind: "rust_lib".to_string(),
            path: Some(cargo_manifest.display().to_string()),
            preview_path: None,
        };
    }
    if let Some(cargo_manifest) = resolve_frontend_host_launcher(manifest, moc_root) {
        return MocDoctorLauncher {
            status: "ok".to_string(),
            kind: "frontend_host".to_string(),
            path: Some(cargo_manifest.display().to_string()),
            preview_path: resolve_frontend_preview(manifest, moc_root)
                .map(|path| path.display().to_string()),
        };
    }
    if let Some(preview_path) = resolve_frontend_lib_preview(moc_root)
        .or_else(|| resolve_frontend_preview(manifest, moc_root))
    {
        return MocDoctorLauncher {
            status: "warn".to_string(),
            kind: "preview_only".to_string(),
            path: None,
            preview_path: Some(preview_path.display().to_string()),
        };
    }

    MocDoctorLauncher {
        status: if manifest.has_validation_flow() {
            "warn".to_string()
        } else {
            "error".to_string()
        },
        kind: if manifest.has_validation_flow() {
            "verify_only".to_string()
        } else {
            "missing".to_string()
        },
        path: None,
        preview_path: None,
    }
}

fn latest_moc_diagnostic(
    blocks_root: &str,
    moc_id: &str,
) -> Result<Option<MocDoctorDiagnostic>, String> {
    let diagnostics_root = resolve_diagnostics_root(blocks_root);
    let events_path = diagnostics_root.join("events.jsonl");
    if !events_path.is_file() {
        return Ok(None);
    }

    let events = read_diagnostic_events(&diagnostics_root)?;
    let selected_trace = events
        .iter()
        .filter(|event| event.moc_id.as_deref() == Some(moc_id))
        .max_by_key(|event| event.timestamp_ms)
        .map(|event| {
            event
                .trace_id
                .clone()
                .unwrap_or_else(|| event.execution_id.clone())
        });
    let Some(trace_id) = selected_trace else {
        return Ok(None);
    };

    let trace_events = events
        .into_iter()
        .filter(|event| {
            event
                .trace_id
                .clone()
                .unwrap_or_else(|| event.execution_id.clone())
                == trace_id
        })
        .collect::<Vec<_>>();
    let failures = trace_events
        .iter()
        .filter(|event| event.event == "block.execution.failure")
        .count();
    let last_error_id = trace_events
        .iter()
        .rev()
        .find_map(|event| event.error_id.clone());
    let mut artifacts = 0;
    for event in &trace_events {
        if read_diagnostic_artifact(&diagnostics_root, &event.execution_id)?.is_some() {
            artifacts += 1;
        }
    }

    Ok(Some(MocDoctorDiagnostic {
        trace_id,
        events: trace_events.len(),
        failures,
        last_error_id,
        artifacts,
    }))
}

fn build_moc_doctor_recommendations(
    manifest: &MocManifest,
    warnings: &[String],
    errors: &[String],
    launcher: &MocDoctorLauncher,
    protocol_health: &MocDoctorProtocolHealth,
    latest_diagnostic: &Option<MocDoctorDiagnostic>,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    if !errors.is_empty() {
        recommendations.push(
            "fix `blocks moc check` errors before attempting runtime verification".to_string(),
        );
    }
    if launcher.status == "error" {
        recommendations.push(
            "add a real launcher or a validation flow so the moc can be executed and diagnosed deterministically"
                .to_string(),
        );
    } else if launcher.kind == "preview_only" {
        recommendations.push(
            "add a real frontend host launcher if this moc should support automated host probes beyond preview-only mode"
                .to_string(),
        );
    }
    if protocol_health.status == "error" {
        recommendations.push(
            "align local `protocols` with dependent `moc.yaml` contracts before composing this moc"
                .to_string(),
        );
    }
    if latest_diagnostic.is_none() && manifest.has_validation_flow() {
        recommendations.push(
            "run `blocks moc verify` once to populate a diagnostic trace for future repair loops"
                .to_string(),
        );
    }
    if latest_diagnostic.is_none() && !manifest.has_validation_flow() {
        recommendations.push(
            "run the real launcher or `blocks moc dev` once to generate runtime diagnostics for this moc"
                .to_string(),
        );
    }
    if warnings
        .iter()
        .any(|warning| warning.contains("preview helper"))
    {
        recommendations.push(
            "check in a preview helper so frontend inspection remains available to humans and AI"
                .to_string(),
        );
    }
    recommendations
}

fn render_moc_doctor_report(report: MocDoctorReport, json_output: bool) -> Result<String, String> {
    if json_output {
        return serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render moc doctor JSON: {error}"));
    }

    let mut lines = vec![
        format!("moc doctor: {}", report.status),
        format!("id: {}", report.moc_id),
        format!("path: {}", report.path),
        format!("check_status: {}", report.check_status),
        format!("descriptor_only: {}", report.descriptor_only),
        format!(
            "launcher: {} ({})",
            report.launcher.kind, report.launcher.status
        ),
        format!(
            "protocol_health: {} ({})",
            report.protocol_health.status, report.protocol_health.summary
        ),
    ];
    if let Some(diagnostic) = &report.latest_diagnostic {
        lines.push(format!(
            "latest_trace: {} events={} failures={} artifacts={}",
            diagnostic.trace_id, diagnostic.events, diagnostic.failures, diagnostic.artifacts
        ));
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
