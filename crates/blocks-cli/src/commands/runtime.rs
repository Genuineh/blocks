use std::fs;
use std::path::{Path, PathBuf};

use blocks_contract::{BlockContract, ContractLoadError};
use blocks_runtime::{
    HostCompatibilityReport, HostProfile, Runtime, RuntimeHost, SyncCliRuntimeHost,
    TokioServiceRuntimeHost,
};
use serde_json::json;

use crate::app::toolchain::{read_text_file, resolve_descriptor_path, resolve_workspace_root};

#[derive(Default)]
struct RuntimeCheckOptions {
    hosts: Vec<HostProfile>,
    json: bool,
}

pub fn run_command(args: &[String]) -> Result<String, String> {
    match args {
        [subcommand, target] if subcommand == "check" => check_command(target, &[]),
        [subcommand, target, rest @ ..] if subcommand == "check" => check_command(target, rest),
        _ => Err(
            "usage: blocks runtime check <block-root|block.yaml> [--host <sync-cli|tokio-service>]... [--json]".to_string(),
        ),
    }
}

pub fn check_command(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_check_options(args)?;
    let (contract, contract_path, _) = load_block_contract(target)?;
    let reports = build_host_reports(&contract, &selected_host_profiles(&options));
    let status = aggregate_status(&reports);

    if options.json {
        let payload = json!({
            "status": status,
            "kind": "runtime",
            "path": contract_path.display().to_string(),
            "block_id": contract.id,
            "hosts": reports.iter().map(report_to_json).collect::<Vec<_>>(),
        });
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|error| format!("failed to render runtime check JSON: {error}"))?;
        return if status == "error" {
            Err(rendered)
        } else {
            Ok(rendered)
        };
    }

    let mut lines = vec![
        format!("runtime check: {status}"),
        format!("id: {}", contract.id),
        format!("path: {}", contract_path.display()),
    ];
    for report in &reports {
        lines.push(format!("host {}: {}", report.host_profile, report.status));
        lines.push(format!(
            "capabilities: model={} in_process={} diagnostics={} trace={} moc={}",
            report.capabilities.runtime_model,
            report.capabilities.in_process,
            report.capabilities.supports_diagnostics_artifacts,
            report.capabilities.supports_trace_context,
            report.capabilities.supports_moc_context
        ));
        for warning in &report.warnings {
            lines.push(format!("warning: {}: {warning}", report.host_profile));
        }
        for error in &report.errors {
            lines.push(format!("error: {}: {error}", report.host_profile));
        }
    }
    let rendered = lines.join("\n");
    if status == "error" {
        Err(rendered)
    } else {
        Ok(rendered)
    }
}

pub(crate) fn load_block_contract(
    target: &str,
) -> Result<(BlockContract, PathBuf, PathBuf), String> {
    let contract_path = resolve_descriptor_path(target, "block.yaml");
    let source = read_text_file(&contract_path, "block contract")?;
    let (contract, _) = match BlockContract::from_yaml_str_with_report(&source) {
        Ok(result) => result,
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
    let block_root = contract_path
        .parent()
        .ok_or_else(|| format!("invalid block contract path: {}", contract_path.display()))?
        .to_path_buf();
    Ok((contract, contract_path, block_root))
}

pub(crate) fn instantiate_hosts(
    profiles: &[HostProfile],
    diagnostics_root: &Path,
) -> Vec<Box<dyn RuntimeHost>> {
    profiles
        .iter()
        .map(|profile| {
            let host_diagnostics_root = diagnostics_root.join(profile.as_str());
            match profile {
                HostProfile::SyncCli => Box::new(SyncCliRuntimeHost::with_runtime(
                    Runtime::with_diagnostics_root(host_diagnostics_root),
                )) as Box<dyn RuntimeHost>,
                HostProfile::TokioService => Box::new(TokioServiceRuntimeHost::with_runtime(
                    Runtime::with_diagnostics_root(host_diagnostics_root),
                )) as Box<dyn RuntimeHost>,
            }
        })
        .collect()
}

pub(crate) fn discover_runtime_input(
    block_root: &Path,
) -> Result<(PathBuf, serde_json::Value), String> {
    let candidates = [block_root.join("fixtures"), block_root.join("examples")];
    for directory in candidates {
        if !directory.is_dir() {
            continue;
        }
        let mut entries = fs::read_dir(&directory)
            .map_err(|error| {
                format!(
                    "failed to read runtime input directory {}: {error}",
                    directory.display()
                )
            })?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .collect::<Vec<_>>();
        entries.sort();
        if let Some(path) = entries.into_iter().next() {
            let value = crate::app::read_json_file(
                path.to_str()
                    .ok_or_else(|| format!("invalid runtime input path: {}", path.display()))?,
            )?;
            return Ok((path, value));
        }
    }

    Err(format!(
        "no runtime input fixture found under {}/fixtures or {}/examples",
        block_root.display(),
        block_root.display()
    ))
}

pub(crate) fn default_runtime_diagnostics_root(block_root: &Path) -> PathBuf {
    resolve_workspace_root(block_root)
        .join(".blocks")
        .join("runtime-hosts")
        .join(
            block_root
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("block"),
        )
}

pub(crate) fn build_host_reports(
    contract: &BlockContract,
    profiles: &[HostProfile],
) -> Vec<HostCompatibilityReport> {
    instantiate_hosts(profiles, Path::new(".blocks/runtime-check"))
        .into_iter()
        .map(|host| host.check_contract(contract))
        .collect()
}

fn parse_check_options(args: &[String]) -> Result<RuntimeCheckOptions, String> {
    let mut options = RuntimeCheckOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--host" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--host requires a value".to_string())?;
                options.hosts.push(HostProfile::parse(value)?);
                index += 2;
            }
            other => return Err(format!("unknown option for runtime check: {other}")),
        }
    }
    Ok(options)
}

fn selected_host_profiles(options: &RuntimeCheckOptions) -> Vec<HostProfile> {
    if options.hosts.is_empty() {
        vec![HostProfile::SyncCli, HostProfile::TokioService]
    } else {
        options.hosts.clone()
    }
}

fn aggregate_status(reports: &[HostCompatibilityReport]) -> &'static str {
    if reports.iter().any(|report| report.status == "error") {
        "error"
    } else if reports.iter().any(|report| report.status == "warn") {
        "warn"
    } else {
        "ok"
    }
}

fn report_to_json(report: &HostCompatibilityReport) -> serde_json::Value {
    json!({
        "host_profile": report.host_profile,
        "status": report.status,
        "warnings": report.warnings,
        "errors": report.errors,
        "capabilities": {
            "runtime_model": report.capabilities.runtime_model,
            "in_process": report.capabilities.in_process,
            "supports_contract_validation": report.capabilities.supports_contract_validation,
            "supports_diagnostics_artifacts": report.capabilities.supports_diagnostics_artifacts,
            "supports_trace_context": report.capabilities.supports_trace_context,
            "supports_moc_context": report.capabilities.supports_moc_context,
        }
    })
}
