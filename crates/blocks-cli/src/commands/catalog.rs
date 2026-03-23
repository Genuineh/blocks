use serde::Serialize;

use blocks_contract::{BlockContract, ImplementationKind, ImplementationTarget};
use blocks_registry::{RegisteredBlock, Registry};

use crate::app::toolchain::count_files;

#[derive(Default)]
struct CatalogOptions {
    json: bool,
    kind: Option<String>,
    target: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CatalogEvidence {
    tests: bool,
    examples: bool,
    evaluators: bool,
    fixtures: bool,
    tests_files: usize,
    examples_files: usize,
    evaluators_files: usize,
    fixtures_files: usize,
}

#[derive(Debug, Clone, Serialize)]
struct CatalogEntry {
    id: String,
    status: String,
    implementation_kind: String,
    implementation_target: String,
    purpose: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    side_effects: Vec<String>,
    evidence: CatalogEvidence,
    contract_path: String,
    implementation_path: String,
    warnings: Vec<String>,
}

pub fn export_command(blocks_root: &str, args: &[String]) -> Result<String, String> {
    let options = parse_catalog_options(args, "catalog export")?;
    let entries = load_catalog_entries(blocks_root)?
        .into_iter()
        .filter(|entry| matches_filters(entry, &options))
        .collect::<Vec<_>>();
    render_catalog_entries("catalog export", &entries, options.json)
}

pub fn search_command(blocks_root: &str, query: &str, args: &[String]) -> Result<String, String> {
    let options = parse_catalog_options(args, "catalog search")?;
    let needle = query.to_ascii_lowercase();
    let entries = load_catalog_entries(blocks_root)?
        .into_iter()
        .filter(|entry| matches_filters(entry, &options))
        .filter(|entry| entry_search_text(entry).contains(&needle))
        .collect::<Vec<_>>();
    render_catalog_entries("catalog search", &entries, options.json)
}

fn parse_catalog_options(args: &[String], label: &str) -> Result<CatalogOptions, String> {
    let mut options = CatalogOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--kind" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--kind requires a value".to_string())?;
                options.kind = Some(value.clone());
                index += 2;
            }
            "--target" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--target requires a value".to_string())?;
                options.target = Some(value.clone());
                index += 2;
            }
            "--status" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--status requires a value".to_string())?;
                options.status = Some(value.clone());
                index += 2;
            }
            other => return Err(format!("unknown option for {label}: {other}")),
        }
    }
    Ok(options)
}

fn load_catalog_entries(blocks_root: &str) -> Result<Vec<CatalogEntry>, String> {
    let registry = Registry::load_from_root(blocks_root).map_err(|error| error.to_string())?;
    Ok(registry
        .list()
        .into_iter()
        .map(build_catalog_entry)
        .collect::<Vec<_>>())
}

fn build_catalog_entry(block: &RegisteredBlock) -> CatalogEntry {
    let implementation = block
        .contract
        .implementation
        .as_ref()
        .expect("registry guarantees implementation metadata");
    let evidence = CatalogEvidence {
        tests: block.block_dir.join("tests").join("run.sh").is_file(),
        examples: block.block_dir.join("examples").join("run.sh").is_file(),
        evaluators: block.block_dir.join("evaluators").join("run.sh").is_file(),
        fixtures: block.block_dir.join("fixtures").exists(),
        tests_files: count_files(&block.block_dir.join("tests")),
        examples_files: count_files(&block.block_dir.join("examples")),
        evaluators_files: count_files(&block.block_dir.join("evaluators")),
        fixtures_files: count_files(&block.block_dir.join("fixtures")),
    };

    CatalogEntry {
        id: block.contract.id.clone(),
        status: block
            .contract
            .status
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        implementation_kind: implementation_kind_name(implementation.kind).to_string(),
        implementation_target: implementation_target_name(implementation.target).to_string(),
        purpose: block.contract.purpose.clone().unwrap_or_default(),
        inputs: schema_summary(&block.contract, true),
        outputs: schema_summary(&block.contract, false),
        side_effects: block.contract.side_effects.clone(),
        evidence,
        contract_path: block.contract_path.display().to_string(),
        implementation_path: block.implementation_path.display().to_string(),
        warnings: block
            .contract_warnings
            .iter()
            .map(|warning| format!("{}: {}", warning.path, warning.message))
            .collect(),
    }
}

fn schema_summary(contract: &BlockContract, input: bool) -> Vec<String> {
    let schema = if input {
        &contract.input_schema
    } else {
        &contract.output_schema
    };
    schema
        .iter()
        .map(|(name, field)| {
            let required = if field.required { "!" } else { "" };
            format!("{name}:{}{}", value_type_name(field.field_type), required)
        })
        .collect()
}

fn matches_filters(entry: &CatalogEntry, options: &CatalogOptions) -> bool {
    if let Some(kind) = &options.kind
        && entry.implementation_kind != kind.as_str()
    {
        return false;
    }
    if let Some(target) = &options.target
        && entry.implementation_target != target.as_str()
    {
        return false;
    }
    if let Some(status) = &options.status
        && entry.status != status.as_str()
    {
        return false;
    }
    true
}

fn entry_search_text(entry: &CatalogEntry) -> String {
    [
        entry.id.as_str(),
        entry.status.as_str(),
        entry.implementation_kind.as_str(),
        entry.implementation_target.as_str(),
        entry.purpose.as_str(),
        &entry.inputs.join(" "),
        &entry.outputs.join(" "),
        &entry.side_effects.join(" "),
    ]
    .join(" ")
    .to_ascii_lowercase()
}

fn render_catalog_entries(
    label: &str,
    entries: &[CatalogEntry],
    json: bool,
) -> Result<String, String> {
    if json {
        return serde_json::to_string_pretty(entries)
            .map_err(|error| format!("failed to render {label} JSON: {error}"));
    }

    let mut lines = vec![format!("{label}: {}", entries.len())];
    for entry in entries {
        lines.push(format!(
            "{} [{}] {}/{} tests={} examples={} evals={} fixtures={}",
            entry.id,
            entry.status,
            entry.implementation_kind,
            entry.implementation_target,
            entry.evidence.tests_files,
            entry.evidence.examples_files,
            entry.evidence.evaluators_files,
            entry.evidence.fixtures_files
        ));
        if !entry.purpose.is_empty() {
            lines.push(format!("  purpose: {}", entry.purpose));
        }
    }
    Ok(lines.join("\n"))
}

fn value_type_name(value_type: blocks_contract::ValueType) -> &'static str {
    match value_type {
        blocks_contract::ValueType::String => "string",
        blocks_contract::ValueType::Number => "number",
        blocks_contract::ValueType::Integer => "integer",
        blocks_contract::ValueType::Boolean => "boolean",
        blocks_contract::ValueType::Object => "object",
        blocks_contract::ValueType::Array => "array",
    }
}

fn implementation_kind_name(kind: ImplementationKind) -> &'static str {
    match kind {
        ImplementationKind::Rust => "rust",
        ImplementationKind::TauriTs => "tauri_ts",
    }
}

fn implementation_target_name(target: ImplementationTarget) -> &'static str {
    match target {
        ImplementationTarget::Backend => "backend",
        ImplementationTarget::Frontend => "frontend",
        ImplementationTarget::Shared => "shared",
    }
}
