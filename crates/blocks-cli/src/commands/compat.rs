use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;

use blocks_bcl::emit_file;
use blocks_contract::{BlockContract, ContractLoadError, FieldSchema, ValueType};
use blocks_moc::{MocManifest, MocProtocol};

use crate::app::toolchain::{read_text_file, resolve_descriptor_path};

#[derive(Default)]
struct CompatOptions {
    json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CompatChange {
    path: String,
    kind: String,
    summary: String,
    severity: String,
}

#[derive(Debug, Clone, Serialize)]
struct CompatReport {
    target_kind: String,
    status: String,
    changes: Vec<CompatChange>,
    summary: String,
}

pub fn run_command(args: &[String]) -> Result<String, String> {
    match args {
        [kind, before, after] if kind == "block" => compare_block(before, after, &[]),
        [kind, before, after, rest @ ..] if kind == "block" => compare_block(before, after, rest),
        [kind, before, after] if kind == "moc" => compare_moc(before, after, &[]),
        [kind, before, after, rest @ ..] if kind == "moc" => compare_moc(before, after, rest),
        [kind, blocks_root, before, after] if kind == "bcl" => {
            compare_bcl(blocks_root, before, after, &[])
        }
        [kind, blocks_root, before, after, rest @ ..] if kind == "bcl" => {
            compare_bcl(blocks_root, before, after, rest)
        }
        _ => Err(
            "usage: blocks compat block <before-block|block.yaml> <after-block|block.yaml> [--json]\n       blocks compat moc <before-moc|moc.yaml> <after-moc|moc.yaml> [--json]\n       blocks compat bcl <blocks-root> <before-moc|moc.bcl> <after-moc|moc.bcl> [--json]"
                .to_string(),
        ),
    }
}

fn compare_block(before: &str, after: &str, args: &[String]) -> Result<String, String> {
    let options = parse_compat_options(args)?;
    let before_path = resolve_descriptor_path(before, "block.yaml");
    let after_path = resolve_descriptor_path(after, "block.yaml");
    let before_contract = load_block_contract(&before_path)?;
    let after_contract = load_block_contract(&after_path)?;

    let mut changes = Vec::new();
    compare_string_field(
        &mut changes,
        "implementation.kind",
        implementation_kind_name(before_contract.implementation.as_ref()),
        implementation_kind_name(after_contract.implementation.as_ref()),
        "breaking",
    );
    compare_string_field(
        &mut changes,
        "implementation.target",
        implementation_target_name(before_contract.implementation.as_ref()),
        implementation_target_name(after_contract.implementation.as_ref()),
        "breaking",
    );
    compare_string_field(
        &mut changes,
        "purpose",
        before_contract.purpose.as_deref(),
        after_contract.purpose.as_deref(),
        "compatible",
    );
    compare_schema_maps(
        &mut changes,
        "input_schema",
        &before_contract.input_schema,
        &after_contract.input_schema,
    );
    compare_schema_maps(
        &mut changes,
        "output_schema",
        &before_contract.output_schema,
        &after_contract.output_schema,
    );

    let report = finalize_report("block", changes);
    render_report(report, options.json)
}

fn compare_moc(before: &str, after: &str, args: &[String]) -> Result<String, String> {
    let options = parse_compat_options(args)?;
    let before_path = resolve_descriptor_path(before, "moc.yaml");
    let after_path = resolve_descriptor_path(after, "moc.yaml");
    let before_manifest = load_moc_manifest(&before_path)?;
    let after_manifest = load_moc_manifest(&after_path)?;

    let mut changes = Vec::new();
    let before_type = before_manifest.moc_type.to_string();
    let after_type = after_manifest.moc_type.to_string();
    let before_backend_mode = before_manifest.backend_mode.map(|mode| mode.to_string());
    let after_backend_mode = after_manifest.backend_mode.map(|mode| mode.to_string());
    compare_string_field(
        &mut changes,
        "type",
        Some(before_type.as_str()),
        Some(after_type.as_str()),
        "breaking",
    );
    compare_string_field(
        &mut changes,
        "language",
        Some(before_manifest.language.as_str()),
        Some(after_manifest.language.as_str()),
        "breaking",
    );
    compare_string_field(
        &mut changes,
        "backend_mode",
        before_backend_mode.as_deref(),
        after_backend_mode.as_deref(),
        "breaking",
    );
    compare_schema_maps(
        &mut changes,
        "public_contract.input_schema",
        &before_manifest.public_contract.input_schema,
        &after_manifest.public_contract.input_schema,
    );
    compare_schema_maps(
        &mut changes,
        "public_contract.output_schema",
        &before_manifest.public_contract.output_schema,
        &after_manifest.public_contract.output_schema,
    );
    compare_protocols(
        &mut changes,
        &before_manifest.protocols,
        &after_manifest.protocols,
    );

    let report = finalize_report("moc", changes);
    render_report(report, options.json)
}

fn compare_bcl(
    blocks_root: &str,
    before: &str,
    after: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_compat_options(args)?;
    let before_path = resolve_descriptor_path(before, "moc.bcl");
    let after_path = resolve_descriptor_path(after, "moc.bcl");
    let before_yaml = emit_file(blocks_root, &before_path.display().to_string())
        .map_err(render_bcl_error)?
        .yaml;
    let after_yaml = emit_file(blocks_root, &after_path.display().to_string())
        .map_err(render_bcl_error)?
        .yaml;
    let before_manifest = MocManifest::from_yaml_str(&before_yaml)
        .map_err(|error| format!("failed to load emitted before manifest: {error}"))?;
    let after_manifest = MocManifest::from_yaml_str(&after_yaml)
        .map_err(|error| format!("failed to load emitted after manifest: {error}"))?;

    let mut changes = Vec::new();
    let before_type = before_manifest.moc_type.to_string();
    let after_type = after_manifest.moc_type.to_string();
    compare_string_field(
        &mut changes,
        "type",
        Some(before_type.as_str()),
        Some(after_type.as_str()),
        "breaking",
    );
    compare_schema_maps(
        &mut changes,
        "public_contract.input_schema",
        &before_manifest.public_contract.input_schema,
        &after_manifest.public_contract.input_schema,
    );
    compare_schema_maps(
        &mut changes,
        "public_contract.output_schema",
        &before_manifest.public_contract.output_schema,
        &after_manifest.public_contract.output_schema,
    );
    compare_protocols(
        &mut changes,
        &before_manifest.protocols,
        &after_manifest.protocols,
    );

    let report = finalize_report("bcl", changes);
    render_report(report, options.json)
}

fn parse_compat_options(args: &[String]) -> Result<CompatOptions, String> {
    let mut options = CompatOptions::default();
    for arg in args {
        match arg.as_str() {
            "--json" => options.json = true,
            other => return Err(format!("unknown option for compat: {other}")),
        }
    }
    Ok(options)
}

fn load_block_contract(path: &Path) -> Result<BlockContract, String> {
    let source = read_text_file(path, "block contract")?;
    match BlockContract::from_yaml_str_with_report(&source) {
        Ok((contract, _)) => Ok(contract),
        Err(ContractLoadError::Parse(error)) => Err(format!(
            "failed to parse block contract {}: {error}",
            path.display()
        )),
        Err(ContractLoadError::InvalidDefinition(message)) => Err(format!(
            "invalid block contract {}: {message}",
            path.display()
        )),
    }
}

fn load_moc_manifest(path: &Path) -> Result<MocManifest, String> {
    let source = read_text_file(path, "moc manifest")?;
    MocManifest::from_yaml_str(&source)
        .map_err(|error| format!("failed to parse moc manifest {}: {error}", path.display()))
}

fn compare_schema_maps(
    changes: &mut Vec<CompatChange>,
    scope: &str,
    before: &BTreeMap<String, FieldSchema>,
    after: &BTreeMap<String, FieldSchema>,
) {
    let field_names = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for field_name in field_names {
        let before_field = before.get(&field_name);
        let after_field = after.get(&field_name);
        match (before_field, after_field) {
            (Some(_), None) => changes.push(CompatChange {
                path: format!("{scope}.{field_name}"),
                kind: "field_removed".to_string(),
                summary: format!("removed field `{field_name}`"),
                severity: "breaking".to_string(),
            }),
            (None, Some(after_field)) => changes.push(CompatChange {
                path: format!("{scope}.{field_name}"),
                kind: "field_added".to_string(),
                summary: format!(
                    "added {}field `{field_name}` ({})",
                    if after_field.required {
                        "required "
                    } else {
                        "optional "
                    },
                    value_type_name(after_field.field_type)
                ),
                severity: if after_field.required {
                    "breaking".to_string()
                } else {
                    "compatible".to_string()
                },
            }),
            (Some(before_field), Some(after_field)) => {
                if before_field.field_type != after_field.field_type {
                    changes.push(CompatChange {
                        path: format!("{scope}.{field_name}.type"),
                        kind: "type_changed".to_string(),
                        summary: format!(
                            "changed `{field_name}` type from {} to {}",
                            value_type_name(before_field.field_type),
                            value_type_name(after_field.field_type)
                        ),
                        severity: "breaking".to_string(),
                    });
                }
                if before_field.required != after_field.required {
                    changes.push(CompatChange {
                        path: format!("{scope}.{field_name}.required"),
                        kind: "required_changed".to_string(),
                        summary: format!(
                            "changed `{field_name}` required from {} to {}",
                            before_field.required, after_field.required
                        ),
                        severity: if after_field.required {
                            "breaking".to_string()
                        } else {
                            "compatible".to_string()
                        },
                    });
                }
            }
            (None, None) => {}
        }
    }
}

fn compare_protocols(
    changes: &mut Vec<CompatChange>,
    before: &[MocProtocol],
    after: &[MocProtocol],
) {
    let before_map = before
        .iter()
        .map(|protocol| (protocol.name.clone(), protocol))
        .collect::<BTreeMap<_, _>>();
    let after_map = after
        .iter()
        .map(|protocol| (protocol.name.clone(), protocol))
        .collect::<BTreeMap<_, _>>();
    let names = before_map
        .keys()
        .chain(after_map.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in names {
        match (before_map.get(&name), after_map.get(&name)) {
            (Some(_), None) => changes.push(CompatChange {
                path: format!("protocols.{name}"),
                kind: "protocol_removed".to_string(),
                summary: format!("removed protocol `{name}`"),
                severity: "breaking".to_string(),
            }),
            (None, Some(_)) => changes.push(CompatChange {
                path: format!("protocols.{name}"),
                kind: "protocol_added".to_string(),
                summary: format!("added protocol `{name}`"),
                severity: "compatible".to_string(),
            }),
            (Some(before_protocol), Some(after_protocol)) => {
                if before_protocol.channel != after_protocol.channel {
                    changes.push(CompatChange {
                        path: format!("protocols.{name}.channel"),
                        kind: "protocol_channel_changed".to_string(),
                        summary: format!(
                            "changed protocol `{name}` channel from {} to {}",
                            before_protocol.channel, after_protocol.channel
                        ),
                        severity: "breaking".to_string(),
                    });
                }
                compare_schema_maps(
                    changes,
                    &format!("protocols.{name}.input_schema"),
                    &before_protocol.input_schema,
                    &after_protocol.input_schema,
                );
                compare_schema_maps(
                    changes,
                    &format!("protocols.{name}.output_schema"),
                    &before_protocol.output_schema,
                    &after_protocol.output_schema,
                );
            }
            (None, None) => {}
        }
    }
}

fn compare_string_field(
    changes: &mut Vec<CompatChange>,
    path: &str,
    before: Option<&str>,
    after: Option<&str>,
    severity: &str,
) {
    if before != after {
        changes.push(CompatChange {
            path: path.to_string(),
            kind: "value_changed".to_string(),
            summary: format!(
                "changed `{path}` from `{}` to `{}`",
                before.unwrap_or("<none>"),
                after.unwrap_or("<none>")
            ),
            severity: severity.to_string(),
        });
    }
}

fn finalize_report(target_kind: &str, mut changes: Vec<CompatChange>) -> CompatReport {
    changes.sort_by(|left, right| left.path.cmp(&right.path));
    let status = if changes.iter().any(|change| change.severity == "breaking") {
        "breaking"
    } else if changes.is_empty() {
        "compatible"
    } else {
        "compatible"
    };
    let summary = if changes.is_empty() {
        "no semantic compatibility changes detected".to_string()
    } else if status == "breaking" {
        format!("detected {} breaking or risky changes", changes.len())
    } else {
        format!(
            "detected {} additive or non-breaking changes",
            changes.len()
        )
    };
    CompatReport {
        target_kind: target_kind.to_string(),
        status: status.to_string(),
        changes,
        summary,
    }
}

fn render_report(report: CompatReport, json: bool) -> Result<String, String> {
    if json {
        return serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to render compat JSON: {error}"));
    }

    let mut lines = vec![
        format!("compat {}: {}", report.target_kind, report.status),
        format!("summary: {}", report.summary),
    ];
    for change in &report.changes {
        lines.push(format!(
            "{} [{}] {}",
            change.path, change.severity, change.summary
        ));
    }
    Ok(lines.join("\n"))
}

fn render_bcl_error(report: blocks_bcl::ValidateReport) -> String {
    let Some(first) = report.rule_results.first() else {
        return "bcl compatibility command failed without diagnostics".to_string();
    };
    format!("{} ({})", first.message, first.rule_id)
}

fn implementation_kind_name(
    implementation: Option<&blocks_contract::BlockImplementation>,
) -> Option<&'static str> {
    implementation.map(|value| match value.kind {
        blocks_contract::ImplementationKind::Rust => "rust",
        blocks_contract::ImplementationKind::TauriTs => "tauri_ts",
    })
}

fn implementation_target_name(
    implementation: Option<&blocks_contract::BlockImplementation>,
) -> Option<&'static str> {
    implementation.map(|value| match value.target {
        blocks_contract::ImplementationTarget::Backend => "backend",
        blocks_contract::ImplementationTarget::Frontend => "frontend",
        blocks_contract::ImplementationTarget::Shared => "shared",
    })
}

fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::String => "string",
        ValueType::Number => "number",
        ValueType::Integer => "integer",
        ValueType::Boolean => "boolean",
        ValueType::Object => "object",
        ValueType::Array => "array",
    }
}
