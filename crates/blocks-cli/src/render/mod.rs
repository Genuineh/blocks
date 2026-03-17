use std::path::Path;

use blocks_runtime::{DiagnosticArtifact, DiagnosticEvent};

pub const USAGE: &str = "usage: blocks <list|show|search> <blocks-root> [query|block-id]\n       blocks run <blocks-root> <block-id> <input-json-file>\n       blocks pkg init <packages-root> --kind <block|moc|bcl> --id <package-id> [--json]\n       blocks pkg resolve <package-root|package.yaml> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--lock] [--json]\n       blocks pkg fetch <package-id> [--provider <workspace:path|file:path|remote:id>]... [--json]\n       blocks pkg publish <package-root|package.yaml> --to <file-registry-path> [--json]\n       blocks runtime check <block-root|block.yaml> [--host <sync-cli|tokio-service>]... [--json]\n       blocks bcl init <moc-root|moc.yaml>\n       blocks bcl fmt <package-root|package.yaml|moc-root|moc.bcl>\n       blocks bcl check <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl validate <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl graph <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl explain <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl build <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--target <runtime-compat|moc-compat>] [--out <path>] [--json]\n       blocks catalog export <blocks-root> [--kind <rust|tauri_ts>] [--target <backend|frontend|shared>] [--status <value>] [--json]\n       blocks catalog search <blocks-root> <query> [--kind <rust|tauri_ts>] [--target <backend|frontend|shared>] [--status <value>] [--json]\n       blocks compat block <before-block|block.yaml> <after-block|block.yaml> [--json]\n       blocks compat moc <before-moc|moc.yaml> <after-moc|moc.yaml> [--json]\n       blocks compat bcl <blocks-root> <before-moc|moc.bcl> <after-moc|moc.bcl> [--json]\n       blocks upgrade block <block-root|block.yaml> [--rule-set r12-phase4-baseline] [--write] [--json]\n       blocks upgrade moc <moc-root|moc.yaml> [--rule-set r12-phase4-baseline] [--write] [--json]\n       blocks upgrade bcl <moc-root|moc.bcl> [--rule-set r12-phase4-baseline] [--write] [--json]\n       blocks conformance run block <block-root|block.yaml> [--json]\n       blocks conformance run package <package-root|package.yaml> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks conformance run runtime <block-root|block.yaml> [--host <sync-cli|tokio-service>]... [--input <json-file>] [--json]\n       blocks conformance run moc <blocks-root> <moc-root|moc.yaml> [--json]\n       blocks conformance run bcl <blocks-root> <moc-root|moc.bcl> [--check-against <moc.yaml>] [--gate-mode <off|warn|error>] [--json]\n       blocks block init <blocks-root> <block-id> [--kind <rust|tauri_ts>] [--target <backend|frontend|shared>]\n       blocks block fmt <block-root|block.yaml>\n       blocks block check <block-root|block.yaml> [--json]\n       blocks block test <block-root|block.yaml> [--json]\n       blocks block eval <block-root|block.yaml> [--json]\n       blocks block diagnose <blocks-root> <block-id> [--latest|--execution-id <id>] [--json]\n       blocks block doctor <blocks-root> <block-id|block-root|block.yaml> [--json]\n       blocks moc init <mocs-root> <moc-id> --type <rust_lib|frontend_lib|frontend_app|backend_app> --language <rust|tauri_ts> [--backend-mode <console|service>]\n       blocks moc init <moc-root> --type <rust_lib|frontend_lib|frontend_app|backend_app> --language <rust|tauri_ts> [--backend-mode <console|service>]\n       blocks moc fmt <moc-root|moc.yaml>\n       blocks moc check <blocks-root> <moc-root|moc.yaml> [--json]\n       blocks moc validate <blocks-root> <moc-yaml>\n       blocks moc doctor <blocks-root> <moc-root|moc.yaml> [--json]\n       blocks moc bcl init <moc-root|moc.yaml>\n       blocks moc bcl fmt <moc-root|moc.bcl>\n       blocks moc bcl check <blocks-root> <moc-root|moc.bcl> [--json]\n       blocks moc bcl validate <blocks-root> <moc.bcl> [--json]\n       blocks moc bcl plan <blocks-root> <moc.bcl> [--json]\n       blocks moc bcl emit <blocks-root> <moc.bcl> [--out <path>] [--check-against <moc.yaml>]\n       blocks moc bcl graph <blocks-root> <moc-root|moc.bcl> [--json]\n       blocks moc bcl explain <blocks-root> <moc-root|moc.bcl> [--json]\n       blocks moc run <blocks-root> <moc-yaml> [input-json-file]\n       blocks moc verify <blocks-root> <moc-yaml> [input-json-file]\n       blocks moc dev <blocks-root> <moc-yaml>\n       blocks moc diagnose <blocks-root> <moc-yaml> [--trace-id <id>] [--json]";

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
