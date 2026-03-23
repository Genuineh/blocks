use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use blocks_contract::{
    ArtifactMode, ArtifactPolicy, BlockContract, BlockImplementation, ContractItem, DebugContract,
    ErrorContract, FailureMode, FieldSchema, ImplementationKind, ImplementationTarget,
    ObserveContract, TaxonomyEntry, ValueType,
};
use blocks_moc::{
    BackendMode, MocContract, MocManifest, MocProtocol, MocType, MocUses, MocVerification,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct CommandRunResult {
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn resolve_descriptor_path(path_or_dir: &str, default_file_name: &str) -> PathBuf {
    let path = PathBuf::from(path_or_dir);
    if path.is_dir() {
        path.join(default_file_name)
    } else {
        path
    }
}

pub fn read_text_file(path: &Path, label: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {label} {}: {error}", path.display()))
}

pub fn write_text_file(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create parent directory {}: {error}",
                parent.display()
            )
        })?;
    }

    fs::write(path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

pub fn ensure_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|error| format!("failed to create directory {}: {error}", path.display()))
}

pub fn ensure_new_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("target path already exists: {}", path.display()));
    }
    ensure_directory(path)
}

pub fn resolve_workspace_root(path: &Path) -> PathBuf {
    let mut current = Some(path);
    while let Some(dir) = current {
        if let Some(name) = dir.file_name().and_then(|value| value.to_str())
            && matches!(name, "blocks" | "mocs")
        {
            return dir.parent().unwrap_or(dir).to_path_buf();
        }
        current = dir.parent();
    }

    path.to_path_buf()
}

pub fn count_files(path: &Path) -> usize {
    fn count(path: &Path) -> usize {
        if !path.exists() {
            return 0;
        }

        let Ok(metadata) = fs::metadata(path) else {
            return 0;
        };
        if metadata.is_file() {
            return 1;
        }
        if !metadata.is_dir() {
            return 0;
        }

        fs::read_dir(path)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.flatten())
            .map(|entry| count(&entry.path()))
            .sum()
    }

    count(path)
}

pub fn run_shell_command(command: &str, cwd: &Path) -> Result<CommandRunResult, String> {
    let output = Command::new("sh")
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .output()
        .map_err(|error| {
            format!(
                "failed to run command `{command}` in {}: {error}",
                cwd.display()
            )
        })?;

    Ok(CommandRunResult {
        command: command.to_string(),
        cwd: cwd.display().to_string(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

pub fn run_shell_script(script_path: &Path, cwd: &Path) -> Result<CommandRunResult, String> {
    let output = Command::new("sh")
        .arg(script_path)
        .current_dir(cwd)
        .output()
        .map_err(|error| {
            format!(
                "failed to run script {} in {}: {error}",
                script_path.display(),
                cwd.display()
            )
        })?;

    Ok(CommandRunResult {
        command: script_path.display().to_string(),
        cwd: cwd.display().to_string(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

pub fn normalize_yaml<T: Serialize>(value: &T, label: &str) -> Result<String, String> {
    let rendered = serde_yaml::to_string(value)
        .map_err(|error| format!("failed to render canonical {label}: {error}"))?;
    Ok(strip_yaml_document_prefix(rendered))
}

pub fn parse_block_kind(raw: &str) -> Result<ImplementationKind, String> {
    match raw {
        "rust" => Ok(ImplementationKind::Rust),
        "tauri_ts" => Ok(ImplementationKind::TauriTs),
        other => Err(format!("unsupported block implementation kind: {other}")),
    }
}

pub fn parse_block_target(raw: &str) -> Result<ImplementationTarget, String> {
    match raw {
        "backend" => Ok(ImplementationTarget::Backend),
        "frontend" => Ok(ImplementationTarget::Frontend),
        "shared" => Ok(ImplementationTarget::Shared),
        other => Err(format!("unsupported block implementation target: {other}")),
    }
}

pub fn validate_block_scaffold_shape(
    kind: ImplementationKind,
    target: ImplementationTarget,
) -> Result<(), String> {
    if kind == ImplementationKind::TauriTs && target != ImplementationTarget::Frontend {
        return Err("tauri_ts blocks must target frontend".to_string());
    }
    Ok(())
}

pub fn parse_moc_type(raw: &str) -> Result<MocType, String> {
    match raw {
        "rust_lib" => Ok(MocType::RustLib),
        "frontend_lib" => Ok(MocType::FrontendLib),
        "frontend_app" => Ok(MocType::FrontendApp),
        "backend_app" => Ok(MocType::BackendApp),
        other => Err(format!("unsupported moc type: {other}")),
    }
}

pub fn parse_backend_mode(raw: &str) -> Result<BackendMode, String> {
    match raw {
        "console" => Ok(BackendMode::Console),
        "service" => Ok(BackendMode::Service),
        other => Err(format!("unsupported backend_mode: {other}")),
    }
}

pub fn validate_moc_scaffold_shape(
    moc_type: MocType,
    language: &str,
    backend_mode: Option<BackendMode>,
) -> Result<(), String> {
    match moc_type {
        MocType::BackendApp => {
            if backend_mode.is_none() {
                return Err("backend_app scaffolds require --backend-mode".to_string());
            }
            if language != "rust" {
                return Err("backend_app scaffolds currently require --language rust".to_string());
            }
        }
        MocType::RustLib => {
            if backend_mode.is_some() {
                return Err("backend_mode is allowed only for backend_app".to_string());
            }
            if language != "rust" {
                return Err("rust_lib scaffolds currently require --language rust".to_string());
            }
        }
        MocType::FrontendLib | MocType::FrontendApp => {
            if backend_mode.is_some() {
                return Err("backend_mode is allowed only for backend_app".to_string());
            }
            if language != "tauri_ts" {
                return Err(
                    "frontend_lib/frontend_app scaffolds currently require --language tauri_ts"
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}

pub fn build_block_contract_template(
    id: &str,
    kind: ImplementationKind,
    target: ImplementationTarget,
) -> BlockContract {
    let implementation_entry = match kind {
        ImplementationKind::Rust => "rust/lib.rs",
        ImplementationKind::TauriTs => "tauri_ts/src/index.ts",
    };
    let owner = match target {
        ImplementationTarget::Frontend => "blocks-frontend-team",
        _ => "blocks-core-team",
    };
    let purpose = match target {
        ImplementationTarget::Frontend => "Provide a single reusable frontend capability.",
        ImplementationTarget::Shared => "Provide a single reusable shared capability.",
        ImplementationTarget::Backend => "Provide a single reusable backend capability.",
    };

    BlockContract {
        id: id.to_string(),
        name: Some(titleize_identifier(id)),
        version: Some("0.1.0".to_string()),
        status: Some("candidate".to_string()),
        owner: Some(owner.to_string()),
        purpose: Some(purpose.to_string()),
        scope: vec!["Deliver one small, explicit capability.".to_string()],
        non_goals: vec!["Composing multiple unrelated responsibilities.".to_string()],
        inputs: vec![ContractItem {
            name: "value".to_string(),
            description: Some("Structured input for the block.".to_string()),
        }],
        preconditions: vec!["Required input fields are present.".to_string()],
        outputs: vec![ContractItem {
            name: "value".to_string(),
            description: Some("Structured output from the block.".to_string()),
        }],
        postconditions: vec!["Output schema validation passes.".to_string()],
        implementation: Some(BlockImplementation {
            kind,
            entry: implementation_entry.to_string(),
            target,
        }),
        dependencies: json!({
            "runtime": ["local-runtime"]
        }),
        side_effects: vec!["Document side effects explicitly before activation.".to_string()],
        timeouts: json!({
            "default_ms": 1000
        }),
        resource_limits: json!({
            "memory_mb": 64
        }),
        failure_modes: vec![
            FailureMode {
                id: "invalid_input".to_string(),
                when: Some("Input payload does not satisfy the declared schema.".to_string()),
            },
            FailureMode {
                id: "internal_error".to_string(),
                when: Some("Implementation fails before producing a valid output.".to_string()),
            },
        ],
        error_codes: vec!["invalid_input".to_string(), "internal_error".to_string()],
        recovery_strategy: vec![
            "Validate the input before invocation.".to_string(),
            "Keep failures diagnosable and retry only when safe.".to_string(),
        ],
        verification: json!({
            "automated": ["add repo-specific verification command here"]
        }),
        evaluation: json!({
            "quality_gates": ["define at least one result-quality gate"]
        }),
        acceptance_criteria: vec!["A valid input produces a schema-valid output.".to_string()],
        debug: Some(DebugContract {
            enabled_in_dev: true,
            emits_structured_logs: true,
            log_fields: vec!["execution_id".to_string(), "trace_id".to_string()],
        }),
        observe: Some(ObserveContract {
            metrics: vec!["execution_total".to_string()],
            emits_failure_artifact: true,
            artifact_policy: Some(ArtifactPolicy {
                mode: ArtifactMode::OnFailure,
                on_failure_minimum: None,
                redaction_profile: None,
                retention: None,
            }),
        }),
        errors: Some(ErrorContract {
            taxonomy: vec![
                TaxonomyEntry {
                    id: "invalid_input".to_string(),
                },
                TaxonomyEntry {
                    id: "internal_error".to_string(),
                },
            ],
        }),
        input_schema: BTreeMap::from([(
            "value".to_string(),
            FieldSchema {
                field_type: ValueType::String,
                required: true,
                min_length: Some(1),
                max_length: None,
                allowed_values: Vec::new(),
            },
        )]),
        output_schema: BTreeMap::from([(
            "value".to_string(),
            FieldSchema {
                field_type: ValueType::String,
                required: true,
                min_length: Some(1),
                max_length: None,
                allowed_values: Vec::new(),
            },
        )]),
    }
}

pub fn build_moc_manifest_template(
    id: &str,
    moc_type: MocType,
    language: &str,
    entry: &str,
    backend_mode: Option<BackendMode>,
) -> MocManifest {
    let uses = MocUses::default();
    let protocols = Vec::<MocProtocol>::new();
    let verification_command = match (moc_type, language) {
        (MocType::BackendApp, "rust") | (MocType::RustLib, "rust") => "cargo test",
        _ => "add verification command here",
    };

    MocManifest {
        id: id.to_string(),
        name: titleize_identifier(id),
        moc_type,
        backend_mode,
        language: language.to_string(),
        entry: entry.to_string(),
        public_contract: MocContract {
            input_schema: BTreeMap::new(),
            output_schema: BTreeMap::new(),
        },
        uses,
        depends_on_mocs: Vec::new(),
        protocols,
        verification: MocVerification {
            commands: vec![verification_command.to_string()],
            entry_flow: None,
            flows: Vec::new(),
        },
        acceptance_criteria: vec![
            "Document the intended capability and replace scaffold placeholders before activation."
                .to_string(),
        ],
    }
}

pub fn titleize_identifier(id: &str) -> String {
    id.split(['-', '.', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut segment = first.to_uppercase().to_string();
                    segment.push_str(chars.as_str());
                    segment
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn sanitize_crate_name(prefix: &str, id: &str) -> String {
    let normalized = id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    format!("{prefix}_{normalized}")
}

pub fn block_readme_template(
    id: &str,
    kind: ImplementationKind,
    target: ImplementationTarget,
) -> String {
    format!(
        "# {title}\n\nThis scaffolded block defines `{id}` as a `{kind:?}` capability targeting `{target:?}`.\n\n## Next Steps\n\n- replace the placeholder purpose, scope, and acceptance criteria in `block.yaml`\n- implement the real capability at the declared entry path\n- add minimal success and failure examples under `examples/` and `fixtures/`\n- wire repository-specific tests and evaluators before promoting the block beyond `candidate`\n",
        title = titleize_identifier(id),
    )
}

pub fn moc_readme_template(id: &str, moc_type: MocType, language: &str) -> String {
    format!(
        "# {title}\n\nThis scaffolded moc defines `{id}` as a `{moc_type}` unit using `{language}`.\n\n## Next Steps\n\n- replace placeholder acceptance criteria in `moc.yaml`\n- implement the real launcher at the declared entry path\n- add the blocks, protocols, and validation flow only after the runtime path is clear\n- document how to verify and run this moc once the scaffold is replaced\n",
        title = titleize_identifier(id),
    )
}

pub fn block_rust_lib_template() -> &'static str {
    "use serde_json::Value;\n\npub fn run(input: &Value) -> Value {\n    input.clone()\n}\n"
}

pub fn block_rust_cargo_toml(id: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n\n[lib]\npath = \"lib.rs\"\n\n[dependencies]\nserde_json = \"1.0\"\n",
        sanitize_crate_name("block", id)
    )
}

pub fn block_tauri_entry_template() -> &'static str {
    "export function run(input) {\n  return input;\n}\n"
}

pub fn moc_backend_main_template(id: &str) -> String {
    format!(
        "fn main() {{\n    println!(\"{id} scaffold: replace backend/src/main.rs with the real launcher\");\n}}\n"
    )
}

pub fn moc_backend_cargo_toml(id: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n\n[[bin]]\nname = \"{}\"\npath = \"src/main.rs\"\n",
        sanitize_crate_name("moc", id),
        sanitize_crate_name("moc", id)
    )
}

pub fn moc_rust_lib_template() -> &'static str {
    "pub fn run() -> &'static str {\n    \"replace src/lib.rs with the real library entry\"\n}\n"
}

pub fn moc_rust_lib_cargo_toml(id: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        sanitize_crate_name("moc", id)
    )
}

pub fn moc_frontend_entry_template(id: &str) -> String {
    format!("export function mount() {{\n  return {{ id: \"{id}\", mounted: true }};\n}}\n")
}

pub fn moc_preview_html_template(id: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <title>{}</title>\n  </head>\n  <body>\n    <main id=\"app\">Replace preview/index.html with the real preview for {}</main>\n  </body>\n</html>\n",
        titleize_identifier(id),
        id
    )
}

fn strip_yaml_document_prefix(rendered: String) -> String {
    if let Some(stripped) = rendered.strip_prefix("---\n") {
        stripped.to_string()
    } else {
        rendered
    }
}
