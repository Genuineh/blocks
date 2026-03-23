use std::fs;

use blocks_package::{read_package_manifest, validate_manifest};
use serde_json::Value;
use tempfile::TempDir;

use blocks_cli::run;

fn write_package_manifest(root: &std::path::Path, body: &str) -> std::path::PathBuf {
    let manifest_path = root.join("package.yaml");
    fs::create_dir_all(root).expect("package root should be created");
    fs::write(&manifest_path, body).expect("package manifest should be written");
    manifest_path
}

fn file_registry_release_root(
    registry_root: &std::path::Path,
    package_id: &str,
    version: &str,
) -> std::path::PathBuf {
    registry_root
        .join(package_id.replace('.', "__"))
        .join(version)
}

fn parse_error_id(error: &str) -> String {
    let payload: Value = serde_json::from_str(error).expect("error should be valid json");
    payload["error_id"]
        .as_str()
        .expect("error_id should exist")
        .to_string()
}

#[test]
fn pkg_init_emits_json_contract_for_manifest_core_validation() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let packages_root = temp_dir.path().join("packages");

    let output = run(vec![
        "pkg".to_string(),
        "init".to_string(),
        packages_root.display().to_string(),
        "--kind".to_string(),
        "block".to_string(),
        "--id".to_string(),
        "demo.phase2".to_string(),
        "--json".to_string(),
    ])
    .expect("pkg init should succeed and emit json contract");

    let payload: Value =
        serde_json::from_str(&output).expect("pkg init output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["kind"], "block");
    assert_eq!(payload["id"], "demo.phase2");
    assert!(payload["created_paths"].is_array());
}

#[test]
fn pkg_resolve_writes_deterministic_lockfile_for_same_graph() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("demo");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.lock
version: 0.1.0
descriptor:
  path: block.yaml
dependencies: []
"#,
    );
    fs::write(package_root.join("block.yaml"), "id: demo.lock\n").expect("descriptor should exist");

    let first = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--lock".to_string(),
        "--json".to_string(),
    ])
    .expect("first pkg resolve should succeed");
    let second = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--lock".to_string(),
        "--json".to_string(),
    ])
    .expect("second pkg resolve should succeed");

    assert_eq!(first, second, "resolve output should be deterministic");
    assert!(package_root.join("blocks.lock").is_file());
}

#[test]
fn pkg_resolve_prefers_workspace_provider_over_file_registry() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let workspace_root = temp_dir.path().join("workspace");
    let file_registry_root = temp_dir.path().join("file-registry");

    let workspace_pkg = workspace_root.join("demo-provider");
    write_package_manifest(
        &workspace_pkg,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.provider
version: 0.2.0
descriptor:
  path: block.yaml
dependencies: []
"#,
    );
    fs::write(workspace_pkg.join("block.yaml"), "id: demo.provider\n")
        .expect("workspace descriptor should exist");
    fs::create_dir_all(&file_registry_root).expect("file registry should exist");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        workspace_pkg.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_registry_root.display()),
        "--json".to_string(),
    ])
    .expect("pkg resolve should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("pkg resolve output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(
        payload["providers"][0],
        format!("workspace:{}", workspace_root.display())
    );
}

#[test]
fn pkg_fetch_reports_phase2_error_contract_without_checksum_field() {
    let package_id = "demo.missing";

    let error = run(vec![
        "pkg".to_string(),
        "fetch".to_string(),
        package_id.to_string(),
        "--json".to_string(),
    ])
    .expect_err("missing package fetch should fail with json error payload");

    let payload: Value =
        serde_json::from_str(&error).expect("pkg fetch error should be valid json");
    assert_eq!(payload["status"], "error");
    let error_id = payload["error_id"]
        .as_str()
        .expect("error_id should be present");
    assert!(
        matches!(
            error_id,
            "pkg.fetch.not_found"
                | "pkg.fetch.source_unavailable"
                | "pkg.fetch.unsupported_source"
                | "pkg.fetch.cache_write_failed"
        ),
        "phase2 fetch error_id should stay in the approved taxonomy"
    );
    assert!(
        !error.contains("checksum"),
        "phase2 fetch errors should not expose checksum mismatch semantics"
    );
}

#[test]
fn pkg_resolve_uses_bridge_for_legacy_root_without_package_manifest() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let legacy_root = temp_dir.path().join("legacy-moc");
    fs::create_dir_all(&legacy_root).expect("legacy root should be created");
    fs::write(
        legacy_root.join("moc.yaml"),
        r#"
id: legacy
name: Legacy Moc
type: rust_lib
language: rust
entry: src/lib.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks: []
  internal_blocks: []
depends_on_mocs: []
protocols: []
acceptance_criteria:
  - bridge mode works
"#,
    )
    .expect("legacy descriptor should be written");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        legacy_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect("pkg resolve should support migration bridge mode");

    let payload: Value =
        serde_json::from_str(&output).expect("pkg resolve output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert!(
        payload["warnings"]
            .as_array()
            .expect("warnings should be present")
            .iter()
            .any(|item| item
                .as_str()
                .is_some_and(|line| line.contains("migration bridge"))),
        "bridge mode should be explicitly reported to the user"
    );
}

#[test]
fn pkg_resolve_falls_back_to_next_provider_when_first_has_no_compatible_candidate() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("demo");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.provider_fallback
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    fs::write(
        package_root.join("block.yaml"),
        "id: demo.provider_fallback\n",
    )
    .expect("descriptor should exist");

    let workspace_root = temp_dir.path().join("workspace-empty");
    let file_registry_root = temp_dir.path().join("file-registry");
    fs::create_dir_all(&workspace_root).expect("workspace provider root should exist");
    fs::create_dir_all(&file_registry_root).expect("file provider root should exist");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--compat".to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_registry_root.display()),
        "--json".to_string(),
    ])
    .expect("pkg resolve should succeed");

    let payload: Value =
        serde_json::from_str(&output).expect("pkg resolve output should be valid json");
    let source_type = payload["resolved"][0]["source"]["type"]
        .as_str()
        .expect("source type should exist");
    assert_eq!(
        source_type, "file",
        "resolver should fall back to file provider when workspace has no compatible candidate"
    );
}

#[test]
fn pkg_resolve_reports_conflicting_release_across_providers() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("demo");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.conflict
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    fs::write(package_root.join("block.yaml"), "id: demo.conflict\n")
        .expect("descriptor should exist");

    let workspace_root = temp_dir.path().join("workspace");
    let file_root = temp_dir.path().join("file");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    fs::create_dir_all(&file_root).expect("file root should exist");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--compat".to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_root.display()),
        "--json".to_string(),
    ])
    .expect_err("resolve should fail when conflicting releases are found");

    let payload: Value = serde_json::from_str(&error).expect("error should be valid json");
    assert_eq!(payload["error_id"], "pkg.resolve.conflicting_release");
}

#[test]
fn pkg_resolve_writes_concrete_dependency_versions_in_lockfile() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("demo");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.lock_versions
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    fs::write(package_root.join("block.yaml"), "id: demo.lock_versions\n")
        .expect("descriptor should exist");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--compat".to_string(),
        "--lock".to_string(),
        "--json".to_string(),
    ])
    .expect("pkg resolve should succeed");
    let payload: Value = serde_json::from_str(&output).expect("output should be valid json");
    let dependency_version = payload["resolved"][0]["dependencies"][0]["version"]
        .as_str()
        .expect("dependency version should exist");
    assert!(
        !dependency_version.contains('^'),
        "lockfile dependencies should store concrete resolved versions, not requirement strings"
    );
}

#[test]
fn pkg_fetch_supports_non_default_release_versions() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let registry_root = temp_dir.path().join("registry");
    let release_root = file_registry_release_root(&registry_root, "demo.fetch", "0.2.4");
    fs::create_dir_all(&release_root).expect("release root should exist");
    fs::write(
        release_root.join("package.yaml"),
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.fetch
version: 0.2.4
descriptor:
  path: block.yaml
dependencies: []
"#,
    )
    .expect("manifest should be present");
    fs::write(release_root.join("block.yaml"), "id: demo.fetch\n")
        .expect("descriptor should exist");

    let output = run(vec![
        "pkg".to_string(),
        "fetch".to_string(),
        "demo.fetch".to_string(),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
        "--json".to_string(),
    ])
    .expect("fetch should support non-0.1.0 versions");
    let payload: Value = serde_json::from_str(&output).expect("fetch output should be valid json");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn strict_manifest_validation_rejects_unknown_top_level_keys() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("strict-unknown");
    let manifest_path = write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.strict_unknown
version: 0.1.0
descriptor:
  path: block.yaml
dependencies: []
unexpected_field: true
"#,
    );
    fs::write(package_root.join("block.yaml"), "id: demo.strict_unknown\n")
        .expect("descriptor should exist");

    let manifest = read_package_manifest(&manifest_path).expect("manifest should parse");
    let validation = validate_manifest(&manifest, &package_root);
    assert!(
        validation.is_err(),
        "strict validation should reject unknown top-level keys"
    );
}

#[test]
fn compat_manifest_validation_warns_unknown_top_level_keys() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("compat-unknown");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.compat_unknown
version: 0.1.0
descriptor:
  path: block.yaml
dependencies: []
unexpected_field: true
"#,
    );
    fs::write(package_root.join("block.yaml"), "id: demo.compat_unknown\n")
        .expect("descriptor should exist");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--compat".to_string(),
        "--json".to_string(),
    ])
    .expect("compat mode should succeed and emit warnings");
    let payload: Value = serde_json::from_str(&output).expect("output should be valid json");
    let warnings = payload["warnings"]
        .as_array()
        .expect("warnings should be an array");
    assert!(
        warnings.iter().any(|item| item
            .as_str()
            .is_some_and(|line| line.contains("unknown key"))),
        "compat mode should keep unknown keys as warnings"
    );
}

#[test]
fn pkg_fetch_returns_human_output_when_json_flag_is_disabled() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let registry_root = temp_dir.path().join("registry");
    let release_root = file_registry_release_root(&registry_root, "demo.human", "0.1.0");
    fs::create_dir_all(&release_root).expect("release root should exist");
    fs::write(
        release_root.join("package.yaml"),
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.human
version: 0.1.0
descriptor:
  path: block.yaml
dependencies: []
"#,
    )
    .expect("manifest should be present");
    fs::write(release_root.join("block.yaml"), "id: demo.human\n")
        .expect("descriptor should exist");

    let output = run(vec![
        "pkg".to_string(),
        "fetch".to_string(),
        "demo.human".to_string(),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
    ])
    .expect("fetch should succeed in non-json mode");
    assert!(
        !output.trim_start().starts_with('{'),
        "non-json mode should return human-readable output"
    );
}

#[test]
fn pkg_resolve_does_not_synthesize_candidates_when_release_is_missing() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.no_synthetic
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.missing
    kind: block
    req: ^0.1.0
"#,
    );
    fs::write(package_root.join("block.yaml"), "id: demo.no_synthetic\n")
        .expect("descriptor should exist");

    let workspace_root = temp_dir.path().join("workspace-primary");
    let file_registry_root = temp_dir.path().join("registry-secondary");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    fs::create_dir_all(&file_registry_root).expect("file registry root should exist");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_registry_root.display()),
        "--json".to_string(),
    ])
    .expect_err("missing release should fail instead of synthesizing a candidate");

    assert_eq!(
        parse_error_id(&error),
        "pkg.resolve.unsatisfied_constraint",
        "resolver should return unsatisfied_constraint when providers have no real release"
    );
}

#[test]
fn pkg_resolve_fallback_is_based_on_real_provider_results_not_path_sentinels() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.real_fallback
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.real
    kind: block
    req: ^0.2.0
"#,
    );
    fs::write(package_root.join("block.yaml"), "id: demo.real_fallback\n")
        .expect("descriptor should exist");

    let workspace_root = temp_dir.path().join("workspace-neutral");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    let registry_root = temp_dir.path().join("file-registry");
    let release_root = file_registry_release_root(&registry_root, "dep.real", "0.2.4");
    fs::create_dir_all(&release_root).expect("release root should exist");
    fs::write(
        release_root.join("package.yaml"),
        r#"
api_version: blocks.pkg/v1
kind: block
id: dep.real
version: 0.2.4
descriptor:
  path: block.yaml
dependencies: []
"#,
    )
    .expect("manifest should exist");
    fs::write(release_root.join("block.yaml"), "id: dep.real\n").expect("descriptor should exist");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
        "--json".to_string(),
    ])
    .expect("resolve should succeed");

    let payload: Value = serde_json::from_str(&output).expect("output should be json");
    assert_eq!(
        payload["resolved"][0]["source"]["type"], "file",
        "resolver should fall back to file provider only when it has a real release"
    );
}

#[test]
fn pkg_resolve_conflict_detection_only_uses_real_releases() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.no_fake_conflict
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.only_file
    kind: block
    req: ^0.1.0
"#,
    );
    fs::write(
        package_root.join("block.yaml"),
        "id: demo.no_fake_conflict\n",
    )
    .expect("descriptor should exist");

    let workspace_root = temp_dir.path().join("workspace-neutral");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    let registry_root = temp_dir.path().join("file-registry");
    let release_root = file_registry_release_root(&registry_root, "dep.only_file", "0.1.0");
    fs::create_dir_all(&release_root).expect("release root should exist");
    fs::write(
        release_root.join("package.yaml"),
        r#"
api_version: blocks.pkg/v1
kind: block
id: dep.only_file
version: 0.1.0
descriptor:
  path: block.yaml
dependencies: []
source:
  type: file
"#,
    )
    .expect("manifest should exist");
    fs::write(release_root.join("block.yaml"), "id: dep.only_file\n")
        .expect("descriptor should exist");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", registry_root.display()),
        "--json".to_string(),
    ])
    .expect("resolver should not report conflict when only one provider has a real release");

    let payload: Value = serde_json::from_str(&output).expect("output should be json");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn pkg_resolve_lockfile_never_uses_req_derived_versions_when_dependency_is_missing() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.no_req_fabrication
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.missing
    kind: block
    req: ^9.9.9
"#,
    );
    fs::write(
        package_root.join("block.yaml"),
        "id: demo.no_req_fabrication\n",
    )
    .expect("descriptor should exist");

    let workspace_root = temp_dir.path().join("workspace-neutral");
    let file_registry_root = temp_dir.path().join("file-registry-neutral");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    fs::create_dir_all(&file_registry_root).expect("file root should exist");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_registry_root.display()),
        "--lock".to_string(),
        "--json".to_string(),
    ])
    .expect_err("resolve should fail without real dependency releases");

    assert_eq!(
        parse_error_id(&error),
        "pkg.resolve.unsatisfied_constraint",
        "resolver should fail instead of fabricating a concrete version from req"
    );
    assert!(
        !package_root.join("blocks.lock").exists(),
        "lockfile should not be written from fabricated dependency versions"
    );
}

fn write_block_fixture(path: &std::path::Path, id: &str) {
    fs::write(path.join("block.yaml"), format!("id: {id}\n")).expect("descriptor should exist");
}

fn write_release_fixture(
    registry_root: &std::path::Path,
    package_id: &str,
    version: &str,
) -> std::path::PathBuf {
    let release_root = file_registry_release_root(registry_root, package_id, version);
    fs::create_dir_all(&release_root).expect("release root should exist");
    fs::write(
        release_root.join("package.yaml"),
        format!(
            "\
api_version: blocks.pkg/v1
kind: block
id: {package_id}
version: {version}
descriptor:
  path: block.yaml
dependencies: []
"
        ),
    )
    .expect("manifest should be present");
    write_block_fixture(&release_root, package_id);
    release_root
}

#[test]
fn pkg_resolve_default_provider_must_not_synthesize_dep_sample() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.default_no_shim
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.default_no_shim");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("default resolve should fail when dep.sample has no real release");

    assert_eq!(parse_error_id(&error), "pkg.resolve.unsatisfied_constraint");
}

#[test]
fn pkg_resolve_missing_dep_sample_must_return_unsatisfied_constraint() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_missing
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_missing");

    let workspace_root = temp_dir.path().join("workspace-nonempty");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--json".to_string(),
    ])
    .expect_err("missing dep.sample should fail");

    assert_eq!(parse_error_id(&error), "pkg.resolve.unsatisfied_constraint");
}

#[test]
fn pkg_resolve_no_lockfile_when_only_dep_sample_shim_would_succeed() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_lock
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_lock");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--lock".to_string(),
        "--json".to_string(),
    ])
    .expect_err("resolve with lock should fail without real releases");

    assert_eq!(parse_error_id(&error), "pkg.resolve.unsatisfied_constraint");
    assert!(
        !package_root.join("blocks.lock").exists(),
        "blocks.lock must not be created when resolution only passes via compat shim"
    );
}

#[test]
fn pkg_resolve_dep_sample_result_must_not_depend_on_provider_path_naming() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_path_name
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_path_name");

    let workspace_a = temp_dir.path().join("workspace-alpha");
    let workspace_b = temp_dir.path().join("workspace-empty-beta");
    fs::create_dir_all(&workspace_a).expect("workspace A should exist");
    fs::create_dir_all(&workspace_b).expect("workspace B should exist");

    let error_a = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_a.display()),
        "--json".to_string(),
    ])
    .expect_err("provider A should fail without real release");
    let error_b = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_b.display()),
        "--json".to_string(),
    ])
    .expect_err("provider B should fail without real release");

    assert_eq!(
        parse_error_id(&error_a),
        "pkg.resolve.unsatisfied_constraint"
    );
    assert_eq!(
        parse_error_id(&error_b),
        "pkg.resolve.unsatisfied_constraint"
    );
}

#[test]
fn pkg_resolve_dep_sample_fallback_requires_real_release_from_next_provider() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_fallback
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_fallback");

    let workspace_root = temp_dir.path().join("workspace-empty");
    let file_root = temp_dir.path().join("file-no-release");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    fs::create_dir_all(&file_root).expect("file root should exist");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_root.display()),
        "--json".to_string(),
    ])
    .expect_err("fallback should fail when next provider has no real release");

    assert_eq!(parse_error_id(&error), "pkg.resolve.unsatisfied_constraint");
}

#[test]
fn pkg_resolve_dep_sample_no_false_conflict_when_only_one_provider_has_real_release() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_no_false_conflict
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_no_false_conflict");

    let workspace_root = temp_dir.path().join("workspace-nonempty");
    let file_root = temp_dir.path().join("file-real");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    fs::create_dir_all(&file_root).expect("file root should exist");
    write_release_fixture(&file_root, "dep.sample", "0.1.5");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_root.display()),
        "--json".to_string(),
    ])
    .expect("resolution should succeed from the provider with a real release");

    let payload: Value = serde_json::from_str(&output).expect("output should be valid json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["resolved"][0]["source"]["type"], "file");
}

#[test]
fn pkg_resolve_dep_sample_conflict_detection_requires_two_real_releases() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_conflict_requires_real
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_conflict_requires_real");

    let workspace_root = temp_dir.path().join("workspace-nonempty");
    let file_root = temp_dir.path().join("file-no-release");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    fs::create_dir_all(&file_root).expect("file root should exist");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_root.display()),
        "--json".to_string(),
    ])
    .expect_err("conflict should not be raised without two real releases");

    assert_eq!(parse_error_id(&error), "pkg.resolve.unsatisfied_constraint");
}

#[test]
fn pkg_resolve_dep_sample_lockfile_version_must_come_from_real_provider_release() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_real_lock
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_real_lock");

    let workspace_root = temp_dir.path().join("workspace-nonempty");
    let file_root = temp_dir.path().join("file-real");
    fs::create_dir_all(&workspace_root).expect("workspace root should exist");
    fs::create_dir_all(&file_root).expect("file root should exist");
    write_release_fixture(&file_root, "dep.sample", "0.1.5");

    let output = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--provider".to_string(),
        format!("workspace:{}", workspace_root.display()),
        "--provider".to_string(),
        format!("file:{}", file_root.display()),
        "--lock".to_string(),
        "--json".to_string(),
    ])
    .expect("resolution should succeed from real file release");

    let payload: Value = serde_json::from_str(&output).expect("output should be valid json");
    assert_eq!(
        payload["resolved"][0]["dependencies"][0]["version"], "0.1.5",
        "dependency version should come from discovered release rather than shim/default"
    );
}

#[test]
fn pkg_resolve_dep_sample_compat_shim_is_default_off() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_default_off
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_default_off");

    let error = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("compat shim must be disabled by default");

    assert_eq!(parse_error_id(&error), "pkg.resolve.unsatisfied_constraint");
}

#[test]
fn pkg_resolve_dep_sample_compat_shim_requires_explicit_opt_in() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let package_root = temp_dir.path().join("root");
    write_package_manifest(
        &package_root,
        r#"
api_version: blocks.pkg/v1
kind: block
id: demo.dep_sample_opt_in
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.sample
    kind: block
    req: ^0.1.0
"#,
    );
    write_block_fixture(&package_root, "demo.dep_sample_opt_in");

    let error_without_compat = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--json".to_string(),
    ])
    .expect_err("without compat opt-in, resolution should fail");
    assert_eq!(
        parse_error_id(&error_without_compat),
        "pkg.resolve.unsatisfied_constraint"
    );

    let output_with_compat = run(vec![
        "pkg".to_string(),
        "resolve".to_string(),
        package_root.display().to_string(),
        "--compat".to_string(),
        "--json".to_string(),
    ])
    .expect("explicit compat opt-in should allow legacy shim behavior");
    let payload: Value = serde_json::from_str(&output_with_compat).expect("output should be json");
    assert_eq!(payload["status"], "ok");
}
