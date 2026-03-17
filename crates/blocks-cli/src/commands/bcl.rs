use std::fs;
use std::path::{Path, PathBuf};

use blocks_bcl::{EmitResult, emit_file};
use blocks_package::PackageKind;
use blocks_registry::PackageProvider;
use serde_json::json;

use super::{moc_bcl, pkg};

#[derive(Default)]
struct BclSharedOptions {
    providers: Vec<PackageProvider>,
    compat: bool,
    json: bool,
}

#[derive(Default)]
struct BclBuildOptions {
    shared: BclSharedOptions,
    out: Option<String>,
    target: Option<String>,
}

struct BclPackageContext {
    package_root: PathBuf,
    package_id: String,
    package_version: String,
    source_path: PathBuf,
    blocks_root: PathBuf,
    resolved_packages: Vec<pkg::ResolvedPackage>,
    warnings: Vec<String>,
}

enum BclExecutionContext {
    Legacy {
        target: String,
        blocks_root: PathBuf,
    },
    Package(BclPackageContext),
}

pub fn run_command(args: &[String]) -> Result<String, String> {
    match args {
        [subcommand, target] if subcommand == "init" => init_command(target, &[]),
        [subcommand, target, rest @ ..] if subcommand == "init" => init_command(target, rest),
        [subcommand, target] if subcommand == "fmt" => fmt_command(target),
        [subcommand, target] if subcommand == "check" => check_command(target, &[]),
        [subcommand, target, rest @ ..] if subcommand == "check" => check_command(target, rest),
        [subcommand, target] if subcommand == "validate" => validate_command(target, &[]),
        [subcommand, target, rest @ ..] if subcommand == "validate" => {
            validate_command(target, rest)
        }
        [subcommand, target] if subcommand == "graph" => graph_command(target, &[]),
        [subcommand, target, rest @ ..] if subcommand == "graph" => graph_command(target, rest),
        [subcommand, target] if subcommand == "explain" => explain_command(target, &[]),
        [subcommand, target, rest @ ..] if subcommand == "explain" => {
            explain_command(target, rest)
        }
        [subcommand, target] if subcommand == "build" => build_command(target, &[]),
        [subcommand, target, rest @ ..] if subcommand == "build" => build_command(target, rest),
        _ => Err(
            "usage: blocks bcl init <moc-root|moc.yaml>\n       blocks bcl fmt <package-root|package.yaml|moc-root|moc.bcl>\n       blocks bcl check <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl validate <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl graph <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl explain <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--json]\n       blocks bcl build <package-root|package.yaml|moc-root|moc.bcl> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--target <runtime-compat|moc-compat>] [--out <path>] [--json]".to_string(),
        ),
    }
}

pub fn init_command(target: &str, args: &[String]) -> Result<String, String> {
    moc_bcl::init_command(target, args)
}

pub fn fmt_command(target: &str) -> Result<String, String> {
    moc_bcl::fmt_command(target)
}

pub fn check_command(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_shared_options(args, "bcl check")?;
    match resolve_execution_context(target, &options)? {
        BclExecutionContext::Legacy {
            target,
            blocks_root,
        } => moc_bcl::check_command(
            &blocks_root.display().to_string(),
            &target,
            &forward_json_flag(options.json),
        ),
        BclExecutionContext::Package(context) => {
            let output = moc_bcl::check_command(
                &context.blocks_root.display().to_string(),
                &context.package_root.display().to_string(),
                &forward_json_flag(options.json),
            )?;
            if options.json {
                Ok(augment_success_json(output, &context)?)
            } else {
                Ok(augment_human_output(output, &context))
            }
        }
    }
}

pub fn validate_command(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_shared_options(args, "bcl validate")?;
    match resolve_execution_context(target, &options)? {
        BclExecutionContext::Legacy {
            target,
            blocks_root,
        } => moc_bcl::validate_command(
            &blocks_root.display().to_string(),
            &target,
            &forward_json_flag(options.json),
        ),
        BclExecutionContext::Package(context) => {
            let output = moc_bcl::validate_command(
                &context.blocks_root.display().to_string(),
                &context.package_root.display().to_string(),
                &forward_json_flag(options.json),
            )?;
            if options.json {
                Ok(augment_success_json(output, &context)?)
            } else {
                Ok(augment_human_output(output, &context))
            }
        }
    }
}

pub fn graph_command(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_shared_options(args, "bcl graph")?;
    match resolve_execution_context(target, &options)? {
        BclExecutionContext::Legacy {
            target,
            blocks_root,
        } => moc_bcl::graph_command(
            &blocks_root.display().to_string(),
            &target,
            &forward_json_flag(options.json),
        ),
        BclExecutionContext::Package(context) => {
            let output = moc_bcl::graph_command(
                &context.blocks_root.display().to_string(),
                &context.package_root.display().to_string(),
                &forward_json_flag(options.json),
            )?;
            if options.json {
                Ok(augment_success_json(output, &context)?)
            } else {
                Ok(augment_human_output(output, &context))
            }
        }
    }
}

pub fn explain_command(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_shared_options(args, "bcl explain")?;
    match resolve_execution_context(target, &options)? {
        BclExecutionContext::Legacy {
            target,
            blocks_root,
        } => moc_bcl::explain_command(
            &blocks_root.display().to_string(),
            &target,
            &forward_json_flag(options.json),
        ),
        BclExecutionContext::Package(context) => {
            let output = moc_bcl::explain_command(
                &context.blocks_root.display().to_string(),
                &context.package_root.display().to_string(),
                &forward_json_flag(options.json),
            )?;
            if options.json {
                Ok(augment_success_json(output, &context)?)
            } else {
                Ok(augment_human_output(output, &context))
            }
        }
    }
}

pub fn build_command(target: &str, args: &[String]) -> Result<String, String> {
    let options = parse_build_options(args)?;
    let lowering_target = options.target.as_deref().unwrap_or("runtime-compat");
    if lowering_target != "runtime-compat" && lowering_target != "moc-compat" {
        return Err(format!("unsupported bcl build target: {lowering_target}"));
    }

    match resolve_execution_context(target, &options.shared)? {
        BclExecutionContext::Legacy {
            target,
            blocks_root,
        } => {
            let emitted = emit_file(&blocks_root.display().to_string(), &target)
                .map_err(moc_bcl::render_report_human)?;
            let output_path = resolve_build_output_path(None, &target, options.out.as_deref())?;
            write_emit_output(&output_path, &emitted)?;
            render_build_result(
                &output_path,
                emitted,
                None,
                &[],
                &[],
                lowering_target,
                options.shared.json,
            )
        }
        BclExecutionContext::Package(context) => {
            let emitted = emit_file(
                &context.blocks_root.display().to_string(),
                &context.source_path.display().to_string(),
            )
            .map_err(moc_bcl::render_report_human)?;
            let output_path = resolve_build_output_path(
                Some(&context.package_root),
                &context.source_path.display().to_string(),
                options.out.as_deref(),
            )?;
            write_emit_output(&output_path, &emitted)?;
            render_build_result(
                &output_path,
                emitted,
                Some((&context.package_id, &context.package_version)),
                &context.resolved_packages,
                &context.warnings,
                lowering_target,
                options.shared.json,
            )
        }
    }
}

fn parse_shared_options(args: &[String], command_label: &str) -> Result<BclSharedOptions, String> {
    let mut options = BclSharedOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--provider" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--provider requires a value".to_string())?;
                options
                    .providers
                    .push(PackageProvider::parse(value).map_err(|error| error.to_string())?);
                index += 2;
            }
            "--compat" => {
                options.compat = true;
                index += 1;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            other => return Err(format!("unknown option for {command_label}: {other}")),
        }
    }
    Ok(options)
}

fn parse_build_options(args: &[String]) -> Result<BclBuildOptions, String> {
    let mut options = BclBuildOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--provider" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--provider requires a value".to_string())?;
                options
                    .shared
                    .providers
                    .push(PackageProvider::parse(value).map_err(|error| error.to_string())?);
                index += 2;
            }
            "--compat" => {
                options.shared.compat = true;
                index += 1;
            }
            "--json" => {
                options.shared.json = true;
                index += 1;
            }
            "--out" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--out requires a value".to_string())?;
                options.out = Some(value.clone());
                index += 2;
            }
            "--target" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--target requires a value".to_string())?;
                options.target = Some(value.clone());
                index += 2;
            }
            other => return Err(format!("unknown option for bcl build: {other}")),
        }
    }
    Ok(options)
}

fn resolve_execution_context(
    target: &str,
    options: &BclSharedOptions,
) -> Result<BclExecutionContext, String> {
    if let Ok((loaded, warnings)) = pkg::load_for_resolve(target, options.compat) {
        if loaded.manifest.kind == PackageKind::Bcl {
            let providers = default_bcl_providers(&loaded.package_root, &options.providers);
            let resolution = pkg::resolve_dependencies(&loaded, &providers, options.compat)?;
            let blocks_root = materialize_package_blocks_root(
                &loaded.package_root,
                &loaded.manifest.id,
                &resolution.packages,
            )?;
            let source_path = loaded.package_root.join(&loaded.manifest.descriptor.path);
            return Ok(BclExecutionContext::Package(BclPackageContext {
                package_root: loaded.package_root,
                package_id: loaded.manifest.id,
                package_version: loaded.manifest.version,
                source_path,
                blocks_root,
                resolved_packages: resolution.packages,
                warnings,
            }));
        }
    }

    let blocks_root = infer_blocks_root(target)?;
    Ok(BclExecutionContext::Legacy {
        target: target.to_string(),
        blocks_root,
    })
}

fn default_bcl_providers(
    package_root: &Path,
    explicit: &[PackageProvider],
) -> Vec<PackageProvider> {
    if !explicit.is_empty() {
        return explicit.to_vec();
    }
    vec![PackageProvider::Workspace(
        package_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| package_root.to_path_buf()),
    )]
}

fn materialize_package_blocks_root(
    package_root: &Path,
    package_id: &str,
    resolved_packages: &[pkg::ResolvedPackage],
) -> Result<PathBuf, String> {
    let blocks_root = package_root
        .join(".blocks")
        .join("bcl-package-context")
        .join(sanitize_id(package_id))
        .join("blocks");
    if blocks_root.exists() {
        fs::remove_dir_all(&blocks_root).map_err(|error| {
            format!(
                "failed to clean package-aware bcl blocks root {}: {error}",
                blocks_root.display()
            )
        })?;
    }
    fs::create_dir_all(&blocks_root).map_err(|error| {
        format!(
            "failed to create package-aware bcl blocks root {}: {error}",
            blocks_root.display()
        )
    })?;

    for package in resolved_packages {
        if package.manifest.kind != PackageKind::Block {
            continue;
        }
        let destination = blocks_root.join(sanitize_id(&package.manifest.id));
        copy_directory_recursive(&package.package_root, &destination)?;
    }

    Ok(blocks_root)
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| {
        format!(
            "failed to create destination directory {}: {error}",
            destination.display()
        )
    })?;
    let entries = fs::read_dir(source).map_err(|error| {
        format!(
            "failed to read source directory {}: {error}",
            source.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read directory entry under {}: {error}",
                source.display()
            )
        })?;
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry_path.is_dir() {
            copy_directory_recursive(&entry_path, &destination_path)?;
        } else {
            fs::copy(&entry_path, &destination_path).map_err(|error| {
                format!(
                    "failed to copy {} to {}: {error}",
                    entry_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn infer_blocks_root(target: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(target);
    let start = if path.is_dir() {
        path
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    for ancestor in start.ancestors() {
        let blocks_root = ancestor.join("blocks");
        if blocks_root.is_dir() {
            return Ok(blocks_root);
        }
    }
    Err(format!(
        "failed to infer blocks root for {target}; use a bcl package root or a workspace layout with a sibling blocks/ directory"
    ))
}

fn resolve_build_output_path(
    package_root: Option<&Path>,
    target: &str,
    explicit_out: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(path) = explicit_out {
        let path = PathBuf::from(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create bcl build output directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        return Ok(path);
    }

    if let Some(package_root) = package_root {
        let output_dir = package_root.join(".blocks").join("bcl-build");
        fs::create_dir_all(&output_dir).map_err(|error| {
            format!(
                "failed to create package bcl build output directory {}: {error}",
                output_dir.display()
            )
        })?;
        return Ok(output_dir.join("moc.yaml"));
    }

    let target_path = PathBuf::from(target);
    let source_parent = if target_path.is_dir() {
        target_path
    } else {
        target_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    let output_dir = source_parent.join(".blocks").join("bcl-build");
    fs::create_dir_all(&output_dir).map_err(|error| {
        format!(
            "failed to create bcl build output directory {}: {error}",
            output_dir.display()
        )
    })?;
    Ok(output_dir.join("moc.yaml"))
}

fn write_emit_output(path: &Path, emitted: &EmitResult) -> Result<(), String> {
    fs::write(path, &emitted.yaml).map_err(|error| {
        format!(
            "failed to write bcl build artifact {}: {error}",
            path.display()
        )
    })
}

fn render_build_result(
    output_path: &Path,
    emitted: EmitResult,
    package_meta: Option<(&str, &str)>,
    resolved_packages: &[pkg::ResolvedPackage],
    warnings: &[String],
    lowering_target: &str,
    as_json: bool,
) -> Result<String, String> {
    if as_json {
        return serde_json::to_string_pretty(&json!({
            "status": "ok",
            "kind": "bcl_build",
            "lowering_target": lowering_target,
            "package": package_meta.map(|(id, version)| json!({
                "id": id,
                "version": version,
            })),
            "resolved_packages": resolved_packages.iter().map(|package| json!({
                "id": package.manifest.id,
                "kind": package.manifest.kind.to_string(),
                "version": package.manifest.version,
                "source": {
                    "type": package.source.source_type,
                    "location": package.source.location,
                },
            })).collect::<Vec<_>>(),
            "artifacts": [{
                "kind": "moc_compat_yaml",
                "path": output_path.display().to_string(),
            }],
            "warnings": warnings,
        }))
        .map_err(|error| format!("failed to render bcl build JSON: {error}"));
    }

    let mut lines = vec![
        "bcl build: ok".to_string(),
        format!("lowering_target: {lowering_target}"),
        format!("artifact: {}", output_path.display()),
    ];
    if let Some((id, version)) = package_meta {
        lines.push(format!("package: {id}@{version}"));
    }
    if !resolved_packages.is_empty() {
        lines.push(format!("resolved_packages: {}", resolved_packages.len()));
    }
    if !warnings.is_empty() {
        lines.extend(warnings.iter().map(|warning| format!("warning: {warning}")));
    }
    lines.push(format!("emitted_bytes: {}", emitted.yaml.len()));
    Ok(lines.join("\n"))
}

fn augment_success_json(output: String, context: &BclPackageContext) -> Result<String, String> {
    let mut payload: serde_json::Value = serde_json::from_str(&output)
        .map_err(|error| format!("failed to parse package-aware bcl JSON: {error}"))?;
    let root = payload
        .as_object_mut()
        .ok_or_else(|| "package-aware bcl JSON must be an object".to_string())?;
    root.insert(
        "package".to_string(),
        json!({
            "id": context.package_id,
            "version": context.package_version,
        }),
    );
    root.insert(
        "resolved_packages".to_string(),
        json!(
            context
                .resolved_packages
                .iter()
                .map(|package| json!({
                    "id": package.manifest.id,
                    "kind": package.manifest.kind.to_string(),
                    "version": package.manifest.version,
                    "source": {
                        "type": package.source.source_type,
                        "location": package.source.location,
                    },
                }))
                .collect::<Vec<_>>()
        ),
    );
    root.insert("warnings".to_string(), json!(context.warnings));
    serde_json::to_string_pretty(&payload)
        .map_err(|error| format!("failed to render package-aware bcl JSON: {error}"))
}

fn augment_human_output(output: String, context: &BclPackageContext) -> String {
    let mut lines = vec![
        format!(
            "package: {}@{}",
            context.package_id, context.package_version
        ),
        format!("resolved_packages: {}", context.resolved_packages.len()),
    ];
    lines.extend(
        context
            .warnings
            .iter()
            .map(|warning| format!("warning: {warning}")),
    );
    lines.push(output);
    lines.join("\n")
}

fn forward_json_flag(enabled: bool) -> Vec<String> {
    if enabled {
        vec!["--json".to_string()]
    } else {
        Vec::new()
    }
}

fn sanitize_id(value: &str) -> String {
    value.replace('.', "__")
}
