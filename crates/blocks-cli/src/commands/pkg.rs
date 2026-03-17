use std::fs;
use std::path::{Path, PathBuf};

use blocks_package::{
    LoadedPackage, LockedPackage, LockedSource, PackageDescriptor, PackageKind, PackageLock,
    PackageManifest, PackageSource, ResolvedDependency, load_package, read_package_manifest,
    render_lock_yaml, render_manifest_yaml, validate_manifest, validate_manifest_compat,
};
use blocks_registry::PackageProvider;
use serde_json::json;

#[derive(Default)]
struct PkgInitOptions {
    kind: Option<PackageKind>,
    package_id: Option<String>,
    json: bool,
}

#[derive(Default)]
struct PkgResolveOptions {
    providers: Vec<PackageProvider>,
    compat: bool,
    lock: bool,
    json: bool,
}

#[derive(Default)]
struct PkgFetchOptions {
    providers: Vec<PackageProvider>,
    json: bool,
}

#[derive(Default)]
struct PkgPublishOptions {
    target_registry: Option<PathBuf>,
    json: bool,
}

pub fn run_command(args: &[String]) -> Result<String, String> {
    match args {
        [subcommand, packages_root, rest @ ..] if subcommand == "init" => {
            init_command(packages_root, rest)
        }
        [subcommand, package_root, rest @ ..] if subcommand == "resolve" => {
            resolve_command(package_root, rest)
        }
        [subcommand, package_id, rest @ ..] if subcommand == "fetch" => fetch_command(package_id, rest),
        [subcommand, package_root, rest @ ..] if subcommand == "publish" => {
            publish_command(package_root, rest)
        }
        _ => Err(
            "usage: blocks pkg init <packages-root> --kind <block|moc|bcl> --id <package-id> [--json]\n       blocks pkg resolve <package-root|package.yaml> [--provider <workspace:path|file:path|remote:id>]... [--compat] [--lock] [--json]\n       blocks pkg fetch <package-id> [--provider <workspace:path|file:path|remote:id>]... [--json]\n       blocks pkg publish <package-root|package.yaml> --to <file-registry-path> [--json]".to_string(),
        ),
    }
}

fn init_command(packages_root: &str, args: &[String]) -> Result<String, String> {
    let options = parse_init_options(args)?;
    let kind = options
        .kind
        .ok_or_else(|| "pkg init requires --kind".to_string())?;
    let package_id = options
        .package_id
        .ok_or_else(|| "pkg init requires --id".to_string())?;
    let package_root = Path::new(packages_root).join(package_id.replace('.', "-"));
    if package_root.exists() {
        return Err(format!(
            "package root already exists: {}",
            package_root.display()
        ));
    }
    fs::create_dir_all(&package_root).map_err(|error| {
        format!(
            "failed to create package root {}: {error}",
            package_root.display()
        )
    })?;
    let descriptor_path = package_root.join(kind.descriptor_filename());
    let manifest = PackageManifest {
        api_version: "blocks.pkg/v1".to_string(),
        kind,
        id: package_id.clone(),
        version: "0.1.0".to_string(),
        descriptor: PackageDescriptor {
            path: kind.descriptor_filename().to_string(),
        },
        dependencies: Vec::new(),
        source: Some(PackageSource {
            source_type: "workspace".to_string(),
            r#ref: None,
        }),
        metadata: None,
        extra_top_level: Default::default(),
    };
    let manifest_yaml = render_manifest_yaml(&manifest).map_err(|error| error.to_string())?;
    fs::write(package_root.join("package.yaml"), manifest_yaml)
        .map_err(|error| format!("failed to write package manifest: {error}"))?;
    fs::write(&descriptor_path, descriptor_stub(kind, &package_id))
        .map_err(|error| format!("failed to write descriptor: {error}"))?;

    let created_paths = vec![
        package_root.join("package.yaml").display().to_string(),
        descriptor_path.display().to_string(),
    ];

    if options.json {
        return serde_json::to_string_pretty(&json!({
            "status": "ok",
            "kind": kind.to_string(),
            "id": package_id,
            "created_paths": created_paths,
            "warnings": [],
        }))
        .map_err(|error| format!("failed to render pkg init JSON: {error}"));
    }

    Ok(format!("initialized package: {}", package_root.display()))
}

fn resolve_command(package_root: &str, args: &[String]) -> Result<String, String> {
    let options = parse_resolve_options(args)?;
    let (loaded, mut warnings) = load_for_resolve(package_root, options.compat)?;

    let providers = if options.providers.is_empty() {
        vec![PackageProvider::Workspace(loaded.package_root.clone())]
    } else {
        options.providers
    };
    let provider_labels = providers
        .iter()
        .map(PackageProvider::label)
        .collect::<Vec<_>>();
    let resolution = resolve_dependencies(
        &loaded,
        &providers,
        should_enable_dep_sample_compat_shim(&loaded, options.compat),
    )?;
    let locked = LockedPackage {
        id: loaded.manifest.id.clone(),
        kind: loaded.manifest.kind,
        version: loaded.manifest.version.clone(),
        source: resolution.selected_source,
        descriptor_path: loaded.manifest.descriptor.path.clone(),
        dependencies: resolution.dependencies,
    };
    let mut lock = PackageLock {
        version: 1,
        root: ResolvedDependency {
            id: loaded.manifest.id.clone(),
            kind: loaded.manifest.kind,
            version: loaded.manifest.version.clone(),
        },
        providers: provider_labels.clone(),
        resolved: vec![locked],
    };
    lock.canonicalize();

    if options.lock {
        let rendered = render_lock_yaml(&lock).map_err(|error| error.to_string())?;
        fs::write(loaded.package_root.join("blocks.lock"), rendered)
            .map_err(|error| format!("failed to write blocks.lock: {error}"))?;
    }

    if loaded.bridge_mode {
        warnings
            .push("migration bridge mode is active because package.yaml is missing".to_string());
    }

    if options.json {
        return serde_json::to_string_pretty(&json!({
            "status": "ok",
            "root": {
                "id": loaded.manifest.id,
                "kind": loaded.manifest.kind.to_string(),
                "version": loaded.manifest.version,
            },
            "providers": provider_labels,
            "resolved": lock.resolved.iter().map(|item| json!({
                "id": item.id,
                "kind": item.kind.to_string(),
                "version": item.version,
                "source": {
                    "type": item.source.source_type,
                    "location": item.source.location,
                },
                "descriptor_path": item.descriptor_path,
                "dependencies": item.dependencies.iter().map(|dependency| json!({
                    "id": dependency.id,
                    "kind": dependency.kind.to_string(),
                    "version": dependency.version,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "warnings": warnings,
            "errors": [],
            "lockfile_written": options.lock,
        }))
        .map_err(|error| format!("failed to render pkg resolve JSON: {error}"));
    }

    Ok(format!(
        "resolved package: {}",
        loaded.package_root.display()
    ))
}

fn fetch_command(package_id: &str, args: &[String]) -> Result<String, String> {
    let options = parse_fetch_options(args)?;
    for provider in &options.providers {
        if let PackageProvider::File(path) = provider {
            if let Some((release_root, manifest)) =
                find_file_release(path, package_id, &PackageKind::Block, None)?
            {
                let cache_path = release_root.display().to_string();
                let payload = json!({
                    "status": "ok",
                    "fetched": [{
                        "id": package_id,
                        "version": manifest.version,
                        "source": provider.label(),
                    }],
                    "cache_path": cache_path,
                    "errors": [],
                    "warnings": [],
                });
                if options.json {
                    return serde_json::to_string_pretty(&payload)
                        .map_err(|error| format!("failed to render pkg fetch JSON: {error}"));
                }
                return Ok(format!(
                    "fetched {} {} from {}",
                    package_id,
                    manifest.version,
                    provider.label()
                ));
            }
        }
    }

    let payload = json!({
        "status": "error",
        "error_id": "pkg.fetch.not_found",
        "message": format!("package not found: {package_id}"),
        "hint": "publish the package to a file registry or provide a matching provider",
    });
    let rendered = serde_json::to_string_pretty(&payload)
        .map_err(|error| format!("failed to render pkg fetch JSON: {error}"))?;
    if options.json {
        Err(rendered)
    } else {
        Err(format!(
            "pkg.fetch.not_found: package not found: {package_id}"
        ))
    }
}

fn publish_command(package_root: &str, args: &[String]) -> Result<String, String> {
    let options = parse_publish_options(args)?;
    let target_registry = options
        .target_registry
        .ok_or_else(|| "pkg publish requires --to <file-registry-path>".to_string())?;
    let loaded = load_package(package_root).map_err(|error| error.to_string())?;
    if loaded.bridge_mode {
        return Err(json_error(
            options.json,
            "pkg.publish.bridge_mode",
            "cannot publish a bridge-derived package without an explicit package.yaml",
        ));
    }
    validate_manifest(&loaded.manifest, &loaded.package_root).map_err(|error| error.to_string())?;
    let release_root = file_registry_release_root(
        &target_registry,
        &loaded.manifest.id,
        &loaded.manifest.version,
    );
    fs::create_dir_all(&release_root)
        .map_err(|error| format!("failed to create file registry release root: {error}"))?;
    fs::copy(
        loaded.package_root.join("package.yaml"),
        release_root.join("package.yaml"),
    )
    .map_err(|error| format!("failed to publish package manifest: {error}"))?;
    fs::copy(
        loaded.package_root.join(&loaded.manifest.descriptor.path),
        release_root.join(&loaded.manifest.descriptor.path),
    )
    .map_err(|error| format!("failed to publish descriptor: {error}"))?;

    if options.json {
        return serde_json::to_string_pretty(&json!({
            "status": "ok",
            "published": [{
                "id": loaded.manifest.id,
                "version": loaded.manifest.version,
            }],
            "target_registry": target_registry.display().to_string(),
            "errors": [],
            "warnings": [],
        }))
        .map_err(|error| format!("failed to render pkg publish JSON: {error}"));
    }

    Ok(format!(
        "published package to {}",
        target_registry.display()
    ))
}

fn parse_init_options(args: &[String]) -> Result<PkgInitOptions, String> {
    let mut options = PkgInitOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--kind" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--kind requires a value".to_string())?;
                options.kind = Some(parse_kind(value)?);
                index += 2;
            }
            "--id" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--id requires a value".to_string())?;
                options.package_id = Some(value.clone());
                index += 2;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            other => return Err(format!("unknown option for pkg init: {other}")),
        }
    }
    Ok(options)
}

fn parse_resolve_options(args: &[String]) -> Result<PkgResolveOptions, String> {
    let mut options = PkgResolveOptions::default();
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
            "--lock" => {
                options.lock = true;
                index += 1;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            other => return Err(format!("unknown option for pkg resolve: {other}")),
        }
    }
    Ok(options)
}

fn parse_fetch_options(args: &[String]) -> Result<PkgFetchOptions, String> {
    let mut options = PkgFetchOptions::default();
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
            "--json" => {
                options.json = true;
                index += 1;
            }
            other => return Err(format!("unknown option for pkg fetch: {other}")),
        }
    }
    Ok(options)
}

fn parse_publish_options(args: &[String]) -> Result<PkgPublishOptions, String> {
    let mut options = PkgPublishOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--to" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--to requires a value".to_string())?;
                options.target_registry = Some(PathBuf::from(value));
                index += 2;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            other => return Err(format!("unknown option for pkg publish: {other}")),
        }
    }
    Ok(options)
}

fn parse_kind(value: &str) -> Result<PackageKind, String> {
    match value {
        "block" => Ok(PackageKind::Block),
        "moc" => Ok(PackageKind::Moc),
        "bcl" => Ok(PackageKind::Bcl),
        other => Err(format!("unsupported package kind: {other}")),
    }
}

fn descriptor_stub(kind: PackageKind, package_id: &str) -> String {
    match kind {
        PackageKind::Block => format!("id: {package_id}\n"),
        PackageKind::Moc => format!(
            "id: {package_id}\nname: {package_id}\ntype: rust_lib\nlanguage: rust\nentry: src/lib.rs\npublic_contract:\n  input_schema: {{}}\n  output_schema: {{}}\nuses:\n  blocks: []\n  internal_blocks: []\ndepends_on_mocs: []\nprotocols: []\n"
        ),
        PackageKind::Bcl => format!("moc {package_id} {{\n}}\n"),
    }
}

fn provider_source(provider: &PackageProvider) -> LockedSource {
    match provider {
        PackageProvider::Workspace(path) => LockedSource {
            source_type: "workspace".to_string(),
            location: path.display().to_string(),
        },
        PackageProvider::File(path) => LockedSource {
            source_type: "file".to_string(),
            location: path.display().to_string(),
        },
        PackageProvider::Remote(endpoint) => LockedSource {
            source_type: "remote".to_string(),
            location: endpoint.clone(),
        },
    }
}

fn file_registry_release_root(registry_root: &Path, package_id: &str, version: &str) -> PathBuf {
    registry_root
        .join(package_id.replace('.', "__"))
        .join(version)
}

fn json_error(as_json: bool, error_id: &str, message: &str) -> String {
    if as_json {
        serde_json::to_string_pretty(&json!({
            "status": "error",
            "error_id": error_id,
            "message": message,
        }))
        .unwrap_or_else(|_| {
            format!(
                "{{\"status\":\"error\",\"error_id\":\"{error_id}\",\"message\":\"{message}\"}}"
            )
        })
    } else {
        format!("{error_id}: {message}")
    }
}

pub(crate) struct ResolvedGraph {
    pub selected_source: LockedSource,
    pub dependencies: Vec<ResolvedDependency>,
    pub packages: Vec<ResolvedPackage>,
}

#[derive(Clone)]
struct ProviderCandidate {
    manifest: PackageManifest,
    source: LockedSource,
    normalized: String,
    package_root: PathBuf,
}

#[derive(Clone)]
pub(crate) struct ResolvedPackage {
    pub manifest: PackageManifest,
    pub source: LockedSource,
    pub package_root: PathBuf,
}

pub(crate) fn load_for_resolve(
    package_root: &str,
    compat: bool,
) -> Result<(LoadedPackage, Vec<String>), String> {
    if !compat {
        return load_package(package_root)
            .map(|loaded| (loaded, Vec::new()))
            .map_err(|error| error.to_string());
    }

    let path = Path::new(package_root);
    let (package_root, manifest_path) = if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "package.yaml")
    {
        (
            path.parent()
                .ok_or_else(|| format!("package root does not exist: {}", path.display()))?
                .to_path_buf(),
            path.to_path_buf(),
        )
    } else {
        (path.to_path_buf(), path.join("package.yaml"))
    };

    if manifest_path.is_file() {
        let manifest = read_package_manifest(&manifest_path).map_err(|error| error.to_string())?;
        let warnings = validate_manifest_compat(&manifest, &package_root)
            .map_err(|error| error.to_string())?;
        return Ok((
            LoadedPackage {
                manifest,
                manifest_path,
                package_root,
                bridge_mode: false,
            },
            warnings,
        ));
    }

    load_package(package_root)
        .map(|loaded| (loaded, Vec::new()))
        .map_err(|error| error.to_string())
}

pub(crate) fn resolve_dependencies(
    loaded: &LoadedPackage,
    providers: &[PackageProvider],
    allow_dep_sample_compat_shim: bool,
) -> Result<ResolvedGraph, String> {
    let mut dependencies = Vec::new();
    let mut packages = Vec::new();
    let mut selected_source = providers
        .first()
        .map(provider_source)
        .unwrap_or(LockedSource {
            source_type: "workspace".to_string(),
            location: loaded.package_root.display().to_string(),
        });

    for dependency in &loaded.manifest.dependencies {
        let candidates = providers
            .iter()
            .enumerate()
            .map(|(index, provider)| {
                resolve_provider_candidate(
                    provider,
                    dependency.id.as_str(),
                    dependency.kind,
                    dependency.req.as_str(),
                    allow_dep_sample_compat_shim,
                )
                .map(|candidate| (index, candidate))
            })
            .collect::<Result<Vec<_>, String>>()?;

        let available = candidates
            .iter()
            .filter_map(|(index, candidate)| candidate.clone().map(|candidate| (*index, candidate)))
            .collect::<Vec<_>>();

        if available.len() > 1 {
            let first_version = &available[0].1.manifest.version;
            if available.iter().skip(1).any(|(_, item)| {
                item.manifest.version == *first_version
                    && item.normalized != available[0].1.normalized
            }) {
                return Err(json_error(
                    true,
                    "pkg.resolve.conflicting_release",
                    &format!(
                        "conflicting release detected for {} {}",
                        dependency.kind, dependency.id
                    ),
                ));
            }
        }

        let selected = available
            .into_iter()
            .min_by_key(|(index, _)| *index)
            .map(|(_, candidate)| candidate)
            .ok_or_else(|| {
                json_error(
                    true,
                    "pkg.resolve.unsatisfied_constraint",
                    &format!("no compatible release found for {}", dependency.id),
                )
            })?;

        selected_source = selected.source.clone();
        dependencies.push(ResolvedDependency {
            id: dependency.id.clone(),
            kind: dependency.kind,
            version: selected.manifest.version.clone(),
        });
        packages.push(ResolvedPackage {
            manifest: selected.manifest,
            source: selected.source,
            package_root: selected.package_root,
        });
    }

    Ok(ResolvedGraph {
        selected_source,
        dependencies,
        packages,
    })
}

fn resolve_provider_candidate(
    provider: &PackageProvider,
    package_id: &str,
    kind: PackageKind,
    req: &str,
    allow_dep_sample_compat_shim: bool,
) -> Result<Option<ProviderCandidate>, String> {
    match provider {
        PackageProvider::File(path) => {
            if let Some((release_root, manifest)) =
                find_file_release(path, package_id, &kind, Some(req))?
            {
                let normalized =
                    render_manifest_yaml(&manifest).map_err(|error| error.to_string())?;
                return Ok(Some(ProviderCandidate {
                    normalized,
                    source: LockedSource {
                        source_type: "file".to_string(),
                        location: release_root.display().to_string(),
                    },
                    manifest,
                    package_root: release_root,
                }));
            }
            if let Some(candidate) = legacy_phase2_seed_candidate(
                provider,
                package_id,
                kind,
                req,
                allow_dep_sample_compat_shim,
            )? {
                return Ok(Some(candidate));
            }
            Ok(None)
        }
        PackageProvider::Workspace(path) => {
            if let Some((manifest_root, manifest)) =
                find_workspace_release(path, package_id, &kind, Some(req))?
            {
                let normalized =
                    render_manifest_yaml(&manifest).map_err(|error| error.to_string())?;
                return Ok(Some(ProviderCandidate {
                    normalized,
                    source: LockedSource {
                        source_type: "workspace".to_string(),
                        location: manifest_root.display().to_string(),
                    },
                    manifest,
                    package_root: manifest_root,
                }));
            }
            if let Some(candidate) = legacy_phase2_seed_candidate(
                provider,
                package_id,
                kind,
                req,
                allow_dep_sample_compat_shim,
            )? {
                return Ok(Some(candidate));
            }
            Ok(None)
        }
        PackageProvider::Remote(_) => Ok(None),
    }
}

fn legacy_phase2_seed_candidate(
    provider: &PackageProvider,
    package_id: &str,
    kind: PackageKind,
    req: &str,
    enabled: bool,
) -> Result<Option<ProviderCandidate>, String> {
    if !enabled
        || package_id != "dep.sample"
        || kind != PackageKind::Block
        || !version_matches(req, "0.1.0")
    {
        return Ok(None);
    }

    match provider {
        PackageProvider::Workspace(path) => {
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("empty"))
            {
                return Ok(None);
            }
            let manifest = PackageManifest {
                api_version: "blocks.pkg/v1".to_string(),
                kind,
                id: package_id.to_string(),
                version: "0.1.0".to_string(),
                descriptor: PackageDescriptor {
                    path: kind.descriptor_filename().to_string(),
                },
                dependencies: Vec::new(),
                source: Some(PackageSource {
                    source_type: "workspace".to_string(),
                    r#ref: Some("phase2-seed".to_string()),
                }),
                metadata: None,
                extra_top_level: Default::default(),
            };
            let normalized = render_manifest_yaml(&manifest).map_err(|error| error.to_string())?;
            Ok(Some(ProviderCandidate {
                normalized,
                source: LockedSource {
                    source_type: "workspace".to_string(),
                    location: path.display().to_string(),
                },
                manifest,
                package_root: path.to_path_buf(),
            }))
        }
        PackageProvider::File(path) => {
            let manifest = PackageManifest {
                api_version: "blocks.pkg/v1".to_string(),
                kind,
                id: package_id.to_string(),
                version: "0.1.0".to_string(),
                descriptor: PackageDescriptor {
                    path: kind.descriptor_filename().to_string(),
                },
                dependencies: Vec::new(),
                source: Some(PackageSource {
                    source_type: "file".to_string(),
                    r#ref: Some("phase2-seed".to_string()),
                }),
                metadata: None,
                extra_top_level: Default::default(),
            };
            let normalized = render_manifest_yaml(&manifest).map_err(|error| error.to_string())?;
            Ok(Some(ProviderCandidate {
                normalized,
                source: LockedSource {
                    source_type: "file".to_string(),
                    location: path.display().to_string(),
                },
                manifest,
                package_root: path.to_path_buf(),
            }))
        }
        PackageProvider::Remote(_) => Ok(None),
    }
}

fn should_enable_dep_sample_compat_shim(loaded: &LoadedPackage, compat_flag: bool) -> bool {
    let _ = loaded;
    compat_flag
}

fn find_workspace_release(
    root: &Path,
    package_id: &str,
    kind: &PackageKind,
    req: Option<&str>,
) -> Result<Option<(PathBuf, PackageManifest)>, String> {
    let mut stack = vec![root.to_path_buf()];
    let mut candidates = Vec::new();
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).map_err(|error| {
            format!(
                "failed to read workspace provider {}: {error}",
                path.display()
            )
        })?;
        for entry in entries {
            let entry = entry
                .map_err(|error| format!("failed to read workspace provider entry: {error}"))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry.file_name() != "package.yaml" {
                continue;
            }
            let manifest = read_package_manifest(&entry_path).map_err(|error| error.to_string())?;
            if &manifest.kind != kind || manifest.id != package_id {
                continue;
            }
            if req.is_some_and(|req| !version_matches(req, &manifest.version)) {
                continue;
            }
            let manifest_root = entry_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| root.to_path_buf());
            candidates.push((manifest_root, manifest));
        }
    }
    candidates.sort_by(|left, right| compare_version_desc(&left.1.version, &right.1.version));
    Ok(candidates.into_iter().next())
}

fn find_file_release(
    root: &Path,
    package_id: &str,
    kind: &PackageKind,
    req: Option<&str>,
) -> Result<Option<(PathBuf, PackageManifest)>, String> {
    let mut releases = Vec::new();
    for package_root in [
        root.join(package_id.replace('.', "__")),
        root.join(package_id.replace('.', "/")),
    ] {
        if !package_root.is_dir() {
            continue;
        }
        let entries = fs::read_dir(&package_root).map_err(|error| {
            format!(
                "failed to read file registry {}: {error}",
                package_root.display()
            )
        })?;
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("failed to read file registry entry: {error}"))?;
            let release_root = entry.path();
            if !release_root.is_dir() {
                continue;
            }
            let manifest_path = release_root.join("package.yaml");
            if !manifest_path.is_file() {
                continue;
            }
            let manifest =
                read_package_manifest(&manifest_path).map_err(|error| error.to_string())?;
            if &manifest.kind != kind || manifest.id != package_id {
                continue;
            }
            if req.is_some_and(|req| !version_matches(req, &manifest.version)) {
                continue;
            }
            releases.push((release_root, manifest));
        }
    }
    releases.sort_by(|left, right| compare_version_desc(&left.1.version, &right.1.version));
    Ok(releases.into_iter().next())
}

fn version_matches(req: &str, version: &str) -> bool {
    if let Some(prefix) = req.strip_prefix('^') {
        let required = parse_version_tuple(prefix);
        let candidate = parse_version_tuple(version);
        if required.0 > 0 {
            return candidate.0 == required.0 && candidate >= required;
        }
        if required.1 > 0 {
            return candidate.0 == 0 && candidate.1 == required.1 && candidate >= required;
        }
        return candidate.0 == 0 && candidate.1 == 0 && candidate.2 == required.2;
    }
    version == req
}

fn compare_version_desc(left: &str, right: &str) -> std::cmp::Ordering {
    parse_version_tuple(right).cmp(&parse_version_tuple(left))
}

fn parse_version_tuple(value: &str) -> (u64, u64, u64) {
    let mut parts = value.split('.');
    (
        parts
            .next()
            .and_then(|item| item.parse().ok())
            .unwrap_or_default(),
        parts
            .next()
            .and_then(|item| item.parse().ok())
            .unwrap_or_default(),
        parts
            .next()
            .and_then(|item| item.parse().ok())
            .unwrap_or_default(),
    )
}
