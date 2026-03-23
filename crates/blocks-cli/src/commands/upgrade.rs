use std::path::PathBuf;

use serde::Serialize;

use blocks_bcl::format_file;
use blocks_contract::{BlockContract, ContractLoadError};
use blocks_moc::MocManifest;

use crate::app::toolchain::{
    ensure_directory, normalize_yaml, read_text_file, resolve_descriptor_path, write_text_file,
};

const PHASE4_RULE_SET: &str = "r12-phase4-baseline";

#[derive(Default)]
struct UpgradeOptions {
    json: bool,
    write: bool,
    rule_set: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct UpgradeReport {
    target_kind: String,
    target: String,
    rule_set: String,
    status: String,
    descriptor_changed: bool,
    created_paths: Vec<String>,
    follow_up_actions: Vec<String>,
    preview: Option<String>,
}

pub fn run_command(args: &[String]) -> Result<String, String> {
    match args {
        [kind, target] if kind == "block" => upgrade_block(target, &[]),
        [kind, target, rest @ ..] if kind == "block" => upgrade_block(target, rest),
        [kind, target] if kind == "moc" => upgrade_moc(target, &[]),
        [kind, target, rest @ ..] if kind == "moc" => upgrade_moc(target, rest),
        [kind, target] if kind == "bcl" => upgrade_bcl(target, &[]),
        [kind, target, rest @ ..] if kind == "bcl" => upgrade_bcl(target, rest),
        _ => Err(
            "usage: blocks upgrade block <block-root|block.yaml> [--rule-set r12-phase4-baseline] [--write] [--json]\n       blocks upgrade moc <moc-root|moc.yaml> [--rule-set r12-phase4-baseline] [--write] [--json]\n       blocks upgrade bcl <moc-root|moc.bcl> [--rule-set r12-phase4-baseline] [--write] [--json]"
                .to_string(),
        ),
    }
}

fn upgrade_block(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_upgrade_options(args)?;
    let contract_path = resolve_descriptor_path(target, "block.yaml");
    let source = read_text_file(&contract_path, "block contract")?;
    let (contract, _) = match BlockContract::from_yaml_str_with_report(&source) {
        Ok(value) => value,
        Err(ContractLoadError::Parse(error)) => {
            return Err(format!(
                "failed to parse block contract {}: {error}",
                contract_path.display()
            ));
        }
        Err(ContractLoadError::InvalidDefinition(message)) => {
            return Err(format!(
                "invalid block contract {}: {message}",
                contract_path.display()
            ));
        }
    };
    let rendered = normalize_yaml(&contract, "block contract")?;
    let block_root = contract_path
        .parent()
        .ok_or_else(|| format!("invalid block contract path: {}", contract_path.display()))?;

    let dirs = [
        block_root.join("tests"),
        block_root.join("examples"),
        block_root.join("evaluators"),
        block_root.join("fixtures"),
    ];
    let created_paths = apply_directory_upgrade(&dirs, options.write)?;
    if options.write && source != rendered {
        write_text_file(&contract_path, &rendered)?;
    }

    let report = UpgradeReport {
        target_kind: "block".to_string(),
        target: contract_path.display().to_string(),
        rule_set: selected_rule_set(&options),
        status: upgrade_status(
            options.write,
            source != rendered || !created_paths.is_empty(),
        ),
        descriptor_changed: source != rendered,
        created_paths,
        follow_up_actions: vec![
            "run `blocks block test` and `blocks block eval` after adding real evidence runners"
                .to_string(),
            "run `blocks conformance run block` before promoting the block".to_string(),
        ],
        preview: (!options.write).then_some(rendered),
    };
    render_upgrade_report(report, options.json)
}

fn upgrade_moc(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_upgrade_options(args)?;
    let manifest_path = resolve_descriptor_path(target, "moc.yaml");
    let source = read_text_file(&manifest_path, "moc manifest")?;
    let manifest = MocManifest::from_yaml_str(&source).map_err(|error| {
        format!(
            "failed to parse moc manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    let rendered = normalize_yaml(&manifest, "moc manifest")?;
    let moc_root = manifest_path
        .parent()
        .ok_or_else(|| format!("invalid moc manifest path: {}", manifest_path.display()))?;
    let dirs = [moc_root.join("tests"), moc_root.join("examples")];
    let created_paths = apply_directory_upgrade(&dirs, options.write)?;
    if options.write && source != rendered {
        write_text_file(&manifest_path, &rendered)?;
    }

    let report = UpgradeReport {
        target_kind: "moc".to_string(),
        target: manifest_path.display().to_string(),
        rule_set: selected_rule_set(&options),
        status: upgrade_status(
            options.write,
            source != rendered || !created_paths.is_empty(),
        ),
        descriptor_changed: source != rendered,
        created_paths,
        follow_up_actions: vec![
            "run `blocks moc check` after replacing scaffold placeholders".to_string(),
            "run `blocks moc doctor` to inspect launcher and diagnostics readiness".to_string(),
        ],
        preview: (!options.write).then_some(rendered),
    };
    render_upgrade_report(report, options.json)
}

fn upgrade_bcl(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_upgrade_options(args)?;
    let source_path = resolve_descriptor_path(target, "moc.bcl");
    let source = read_text_file(&source_path, "moc bcl source")?;
    let rendered = format_file(&source_path.display().to_string()).map_err(render_bcl_error)?;
    if options.write && source != rendered {
        write_text_file(&source_path, &rendered)?;
    }

    let report = UpgradeReport {
        target_kind: "bcl".to_string(),
        target: source_path.display().to_string(),
        rule_set: selected_rule_set(&options),
        status: upgrade_status(options.write, source != rendered),
        descriptor_changed: source != rendered,
        created_paths: Vec::new(),
        follow_up_actions: vec![
            "run `blocks moc bcl check` after reviewing the canonical rewrite".to_string(),
            "run `blocks conformance run bcl --check-against` before enabling stricter gate modes"
                .to_string(),
        ],
        preview: (!options.write).then_some(rendered),
    };
    render_upgrade_report(report, options.json)
}

fn parse_upgrade_options(args: &[String]) -> Result<UpgradeOptions, String> {
    let mut options = UpgradeOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--write" => {
                options.write = true;
                index += 1;
            }
            "--rule-set" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--rule-set requires a value".to_string())?;
                if value != PHASE4_RULE_SET {
                    return Err(format!("unsupported upgrade rule set: {value}"));
                }
                options.rule_set = Some(value.clone());
                index += 2;
            }
            other => return Err(format!("unknown option for upgrade: {other}")),
        }
    }
    Ok(options)
}

fn apply_directory_upgrade(paths: &[PathBuf], write: bool) -> Result<Vec<String>, String> {
    let mut created = Vec::new();
    for path in paths {
        if !path.exists() {
            if write {
                ensure_directory(path)?;
            }
            created.push(path.display().to_string());
        }
    }
    Ok(created)
}

fn selected_rule_set(options: &UpgradeOptions) -> String {
    options
        .rule_set
        .clone()
        .unwrap_or_else(|| PHASE4_RULE_SET.to_string())
}

fn upgrade_status(write: bool, changed: bool) -> String {
    if !changed {
        "no_change".to_string()
    } else if write {
        "updated".to_string()
    } else {
        "preview".to_string()
    }
}

fn render_upgrade_report(report: UpgradeReport, json: bool) -> Result<String, String> {
    if json {
        return serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render upgrade JSON: {error}"));
    }

    let mut lines = vec![
        format!("upgrade {}: {}", report.target_kind, report.status),
        format!("target: {}", report.target),
        format!("rule_set: {}", report.rule_set),
        format!("descriptor_changed: {}", report.descriptor_changed),
    ];
    for path in &report.created_paths {
        lines.push(format!("create: {path}"));
    }
    for action in &report.follow_up_actions {
        lines.push(format!("next: {action}"));
    }
    Ok(lines.join("\n"))
}

fn render_bcl_error(report: blocks_bcl::ValidateReport) -> String {
    let Some(first) = report.rule_results.first() else {
        return "bcl upgrade failed without diagnostics".to_string();
    };
    format!("{} ({})", first.message, first.rule_id)
}
