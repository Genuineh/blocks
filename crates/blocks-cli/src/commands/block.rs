use blocks_runtime::{DiagnosticEvent, read_diagnostic_artifact, read_diagnostic_events};
use serde_json::json;

use crate::app::resolve_diagnostics_root;
use crate::render::render_block_diagnose_human;

#[derive(Default)]
struct BlockDiagnoseOptions {
    execution_id: Option<String>,
    json: bool,
}

pub fn diagnose_command(
    blocks_root: &str,
    block_id: &str,
    args: &[String],
) -> Result<String, String> {
    let options = parse_diagnose_options(args)?;
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

fn parse_diagnose_options(args: &[String]) -> Result<BlockDiagnoseOptions, String> {
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

fn select_latest_execution_id(events: &[DiagnosticEvent]) -> Option<String> {
    events
        .iter()
        .max_by_key(|event| event.timestamp_ms)
        .map(|event| event.execution_id.clone())
}
