use std::collections::BTreeMap;
use std::fmt::Write;

use blocks_contract::{FieldSchema, ValueType};
use blocks_moc::{BackendMode, MocManifest, MocProtocol, MocType};

use crate::ir::{GuardSpan, RecoverSpan};

pub fn canonical_yaml(manifest: &MocManifest) -> Result<String, String> {
    serde_yaml::to_string(manifest)
        .map_err(|error| format!("failed to emit canonical moc yaml: {error}"))
}

pub fn canonical_bcl(manifest: &MocManifest) -> Result<String, String> {
    canonical_bcl_with_metadata(manifest, None, None)
}

pub fn canonical_bcl_with_metadata(
    manifest: &MocManifest,
    guard_clauses: Option<&BTreeMap<(String, String), GuardSpan>>,
    recover_clauses: Option<&BTreeMap<String, RecoverSpan>>,
) -> Result<String, String> {
    let mut rendered = String::new();
    writeln!(&mut rendered, "moc {} {{", manifest.id)
        .map_err(|error| format!("failed to render BCL header: {error}"))?;
    writeln!(
        &mut rendered,
        "  name \"{}\";",
        escape_bcl_string(&manifest.name)
    )
    .map_err(|error| format!("failed to render BCL name: {error}"))?;
    writeln!(&mut rendered, "  type {};", render_moc_type(manifest))
        .map_err(|error| format!("failed to render BCL type: {error}"))?;
    writeln!(&mut rendered, "  language {};", manifest.language.trim())
        .map_err(|error| format!("failed to render BCL language: {error}"))?;
    writeln!(
        &mut rendered,
        "  entry \"{}\";",
        escape_bcl_string(&manifest.entry)
    )
    .map_err(|error| format!("failed to render BCL entry: {error}"))?;
    render_schema_section(
        &mut rendered,
        "input",
        &manifest.public_contract.input_schema,
    )?;
    render_schema_section(
        &mut rendered,
        "output",
        &manifest.public_contract.output_schema,
    )?;
    render_uses_section(&mut rendered, manifest)?;
    render_dependencies_section(&mut rendered, manifest)?;
    render_protocols_section(&mut rendered, manifest)?;
    render_verification_section(&mut rendered, manifest)?;
    for acceptance in &manifest.acceptance_criteria {
        writeln!(
            &mut rendered,
            "  accept \"{}\";",
            escape_bcl_string(acceptance)
        )
        .map_err(|error| format!("failed to render BCL acceptance criteria: {error}"))?;
    }
    rendered.push_str("}\n");
    Ok(rendered)
}

pub fn check_parity(emitted_yaml: &str, against_source: &str) -> Result<(), String> {
    let emitted_manifest = MocManifest::from_yaml_str(emitted_yaml)
        .map_err(|error| format!("failed to parse emitted moc yaml: {error}"))?;
    let against_manifest = MocManifest::from_yaml_str(against_source)
        .map_err(|error| format!("failed to parse check-against moc yaml: {error}"))?;

    let emitted_canonical = canonical_yaml(&emitted_manifest)?;
    let against_canonical = canonical_yaml(&against_manifest)?;
    if emitted_canonical == against_canonical {
        Ok(())
    } else {
        Err(
            "emitted moc does not match the check-against manifest after canonical normalization"
                .to_string(),
        )
    }
}

fn render_schema_section(
    rendered: &mut String,
    name: &str,
    fields: &std::collections::BTreeMap<String, FieldSchema>,
) -> Result<(), String> {
    if fields.is_empty() {
        writeln!(rendered, "  {name} {{ }}")
            .map_err(|error| format!("failed to render BCL {name} section: {error}"))?;
        return Ok(());
    }

    writeln!(rendered, "  {name} {{")
        .map_err(|error| format!("failed to render BCL {name} section: {error}"))?;
    for (field_name, field_schema) in fields {
        let required = if field_schema.required {
            " required"
        } else {
            ""
        };
        writeln!(
            rendered,
            "    {field_name}: {}{required};",
            render_value_type(field_schema.field_type)
        )
        .map_err(|error| format!("failed to render BCL {name} field: {error}"))?;
    }
    writeln!(rendered, "  }}")
        .map_err(|error| format!("failed to render BCL {name} section: {error}"))?;
    Ok(())
}

fn render_uses_section(rendered: &mut String, manifest: &MocManifest) -> Result<(), String> {
    if manifest.uses.blocks.is_empty() && manifest.uses.internal_blocks.is_empty() {
        writeln!(rendered, "  uses {{ }}")
            .map_err(|error| format!("failed to render BCL uses section: {error}"))?;
        return Ok(());
    }

    writeln!(rendered, "  uses {{")
        .map_err(|error| format!("failed to render BCL uses section: {error}"))?;
    for block_id in &manifest.uses.blocks {
        writeln!(rendered, "    block {block_id};")
            .map_err(|error| format!("failed to render BCL uses block: {error}"))?;
    }
    for block_id in &manifest.uses.internal_blocks {
        writeln!(rendered, "    internal_block {block_id};")
            .map_err(|error| format!("failed to render BCL internal block: {error}"))?;
    }
    writeln!(rendered, "  }}")
        .map_err(|error| format!("failed to render BCL uses section: {error}"))?;
    Ok(())
}

fn render_dependencies_section(
    rendered: &mut String,
    manifest: &MocManifest,
) -> Result<(), String> {
    if manifest.depends_on_mocs.is_empty() {
        writeln!(rendered, "  depends_on_mocs {{ }}")
            .map_err(|error| format!("failed to render BCL depends_on_mocs section: {error}"))?;
        return Ok(());
    }

    writeln!(rendered, "  depends_on_mocs {{")
        .map_err(|error| format!("failed to render BCL depends_on_mocs section: {error}"))?;
    for dependency in &manifest.depends_on_mocs {
        writeln!(
            rendered,
            "    moc \"{}\" via {};",
            escape_bcl_string(&dependency.moc),
            dependency.protocol
        )
        .map_err(|error| format!("failed to render BCL dependency: {error}"))?;
    }
    writeln!(rendered, "  }}")
        .map_err(|error| format!("failed to render BCL depends_on_mocs section: {error}"))?;
    Ok(())
}

fn render_protocols_section(rendered: &mut String, manifest: &MocManifest) -> Result<(), String> {
    if manifest.protocols.is_empty() {
        writeln!(rendered, "  protocols {{ }}")
            .map_err(|error| format!("failed to render BCL protocols section: {error}"))?;
        return Ok(());
    }

    writeln!(rendered, "  protocols {{")
        .map_err(|error| format!("failed to render BCL protocols section: {error}"))?;
    for protocol in &manifest.protocols {
        render_protocol(rendered, protocol)?;
    }
    writeln!(rendered, "  }}")
        .map_err(|error| format!("failed to render BCL protocols section: {error}"))?;
    Ok(())
}

fn render_protocol(rendered: &mut String, protocol: &MocProtocol) -> Result<(), String> {
    writeln!(rendered, "    protocol {} {{", protocol.name)
        .map_err(|error| format!("failed to render BCL protocol: {error}"))?;
    writeln!(rendered, "      channel {};", protocol.channel)
        .map_err(|error| format!("failed to render BCL protocol channel: {error}"))?;
    render_nested_schema_section(rendered, "input", &protocol.input_schema)?;
    render_nested_schema_section(rendered, "output", &protocol.output_schema)?;
    writeln!(rendered, "    }}")
        .map_err(|error| format!("failed to render BCL protocol: {error}"))?;
    Ok(())
}

fn render_nested_schema_section(
    rendered: &mut String,
    name: &str,
    fields: &std::collections::BTreeMap<String, FieldSchema>,
) -> Result<(), String> {
    if fields.is_empty() {
        writeln!(rendered, "      {name} {{ }}")
            .map_err(|error| format!("failed to render nested BCL {name} section: {error}"))?;
        return Ok(());
    }

    writeln!(rendered, "      {name} {{")
        .map_err(|error| format!("failed to render nested BCL {name} section: {error}"))?;
    for (field_name, field_schema) in fields {
        let required = if field_schema.required {
            " required"
        } else {
            ""
        };
        writeln!(
            rendered,
            "        {field_name}: {}{required};",
            render_value_type(field_schema.field_type)
        )
        .map_err(|error| format!("failed to render nested BCL field: {error}"))?;
    }
    writeln!(rendered, "      }}")
        .map_err(|error| format!("failed to render nested BCL {name} section: {error}"))?;
    Ok(())
}

fn render_verification_section(
    rendered: &mut String,
    manifest: &MocManifest,
) -> Result<(), String> {
    let verification = &manifest.verification;
    if verification.commands.is_empty() && verification.flows.is_empty() {
        writeln!(rendered, "  verification {{ }}")
            .map_err(|error| format!("failed to render BCL verification section: {error}"))?;
        return Ok(());
    }

    writeln!(rendered, "  verification {{")
        .map_err(|error| format!("failed to render BCL verification section: {error}"))?;
    for command in &verification.commands {
        writeln!(rendered, "    command \"{}\";", escape_bcl_string(command))
            .map_err(|error| format!("failed to render BCL verification command: {error}"))?;
    }
    for flow in &verification.flows {
        let entry_prefix = if verification.entry_flow.as_deref() == Some(flow.id.as_str()) {
            "entry "
        } else {
            ""
        };
        writeln!(rendered, "    {entry_prefix}flow {} {{", flow.id)
            .map_err(|error| format!("failed to render BCL flow: {error}"))?;
        for step in &flow.steps {
            writeln!(rendered, "      step {} = {};", step.id, step.block)
                .map_err(|error| format!("failed to render BCL flow step: {error}"))?;
        }
        for bind in &flow.binds {
            writeln!(rendered, "      bind {} -> {};", bind.from, bind.to)
                .map_err(|error| format!("failed to render BCL flow bind: {error}"))?;
        }
        writeln!(rendered, "    }}")
            .map_err(|error| format!("failed to render BCL flow: {error}"))?;
    }
    writeln!(rendered, "  }}")
        .map_err(|error| format!("failed to render BCL verification section: {error}"))?;
    Ok(())
}

fn render_moc_type(manifest: &MocManifest) -> String {
    match (manifest.moc_type, manifest.backend_mode) {
        (MocType::BackendApp, Some(BackendMode::Console)) => "backend_app(console)".to_string(),
        (MocType::BackendApp, Some(BackendMode::Service)) => "backend_app(service)".to_string(),
        (moc_type, _) => moc_type.to_string(),
    }
}

fn render_value_type(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::String => "string",
        ValueType::Number => "number",
        ValueType::Integer => "integer",
        ValueType::Boolean => "boolean",
        ValueType::Object => "object",
        ValueType::Array => "array",
    }
}

fn escape_bcl_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
