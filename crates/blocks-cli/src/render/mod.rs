use std::path::Path;

use blocks_runtime::{DiagnosticArtifact, DiagnosticEvent};

pub const USAGE: &str = "usage: blocks <list|show|search> <blocks-root> [query|block-id]\n       blocks run <blocks-root> <block-id> <input-json-file>\n       blocks block diagnose <blocks-root> <block-id> [--latest|--execution-id <id>] [--json]\n       blocks moc validate <blocks-root> <moc-yaml>\n       blocks moc run <blocks-root> <moc-yaml> [input-json-file]\n       blocks moc verify <blocks-root> <moc-yaml> [input-json-file]\n       blocks moc dev <blocks-root> <moc-yaml>\n       blocks moc diagnose <blocks-root> <moc-yaml> [--trace-id <id>] [--json]";

pub fn render_block_diagnose_human(
    block_id: &str,
    execution_id: &str,
    diagnostics_root: &Path,
    events: &[DiagnosticEvent],
    artifact: Option<&DiagnosticArtifact>,
) -> Result<String, String> {
    let last_event = events
        .last()
        .ok_or_else(|| format!("no diagnostics found for block {block_id}"))?;
    let status = if last_event.event == "block.execution.failure" {
        "failure"
    } else {
        "success"
    };
    let mut lines = vec![
        format!("block: {block_id}"),
        format!("execution_id: {execution_id}"),
        format!("status: {status}"),
        format!("events: {}", events.len()),
        format!("diagnostics_root: {}", diagnostics_root.display()),
    ];
    if let Some(trace_id) = &last_event.trace_id {
        lines.push(format!("trace_id: {trace_id}"));
    }
    if let Some(duration_ms) = last_event.duration_ms {
        lines.push(format!("duration_ms: {duration_ms}"));
    }
    if let Some(error_id) = &last_event.error_id {
        lines.push(format!("error_id: {error_id}"));
    }
    if artifact.is_some() {
        lines.push(format!(
            "artifact: {}/artifacts/{}.json",
            diagnostics_root.display(),
            execution_id
        ));
    }
    Ok(lines.join("\n"))
}

pub fn render_moc_diagnose_human(
    moc_id: &str,
    trace_id: &str,
    diagnostics_root: &Path,
    events: &[DiagnosticEvent],
    artifacts: &[DiagnosticArtifact],
) -> Result<String, String> {
    let first_event = events
        .first()
        .ok_or_else(|| format!("no diagnostics found for trace_id {trace_id}"))?;
    let last_event = events
        .last()
        .ok_or_else(|| format!("no diagnostics found for trace_id {trace_id}"))?;
    let total_duration = last_event
        .timestamp_ms
        .saturating_sub(first_event.timestamp_ms);
    let failure_count = events
        .iter()
        .filter(|event| event.event == "block.execution.failure")
        .count();

    let mut lines = vec![
        format!("moc: {moc_id}"),
        format!("trace_id: {trace_id}"),
        format!("events: {}", events.len()),
        format!("failures: {failure_count}"),
        format!("duration_ms: {total_duration}"),
        format!("diagnostics_root: {}", diagnostics_root.display()),
    ];
    if !artifacts.is_empty() {
        lines.push(format!("artifacts: {}", artifacts.len()));
    }
    Ok(lines.join("\n"))
}

pub fn render_browser_preview_lines(preview_path: &Path, workspace_root: &Path) -> Vec<String> {
    const DEFAULT_PORT: u16 = 4173;

    let preview_target = preview_path
        .strip_prefix(workspace_root)
        .unwrap_or(preview_path);
    let mut browser_path = preview_target.to_string_lossy().replace('\\', "/");
    if !browser_path.starts_with('/') {
        browser_path.insert(0, '/');
    }

    vec![
        format!(
            "browser preview: python3 -m http.server --directory {} {DEFAULT_PORT}",
            shell_quote_path(workspace_root)
        ),
        format!("browser url: http://127.0.0.1:{DEFAULT_PORT}{browser_path}"),
    ]
}

fn shell_quote_path(path: &Path) -> String {
    let rendered = path.display().to_string();
    if rendered
        .chars()
        .any(|ch| ch.is_whitespace() || ch == '\'' || ch == '"')
    {
        format!("'{}'", rendered.replace('\'', "'\\''"))
    } else {
        rendered
    }
}
