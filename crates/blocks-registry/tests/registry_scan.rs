use std::fs;
use std::sync::{Mutex, OnceLock};

use blocks_contract::ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV;
use blocks_registry::{Registry, RegistryError};
use tempfile::TempDir;

fn env_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn write_contract(root: &TempDir, dir_name: &str, id: &str, name: &str) {
    write_contract_with_extra(root, dir_name, id, name, "");
}

fn write_contract_with_extra(root: &TempDir, dir_name: &str, id: &str, name: &str, extra: &str) {
    let block_dir = root.path().join(dir_name);
    let rust_dir = block_dir.join("rust");
    fs::create_dir_all(&rust_dir).expect("block dir should be created");
    fs::write(
        block_dir.join("block.yaml"),
        format!(
            "id: {id}\nname: {name}\nversion: 0.1.0\nstatus: candidate\nowner: blocks-core-team\npurpose: test contract\nscope:\n  - test scope\nnon_goals:\n  - test non-goal\ninputs:\n  - name: value\n    description: input\ninput_schema:\n  value:\n    type: string\n    required: true\npreconditions:\n  - input exists\noutputs:\n  - name: value\n    description: output\noutput_schema:\n  value:\n    type: string\n    required: true\npostconditions:\n  - output exists\nimplementation:\n  kind: rust\n  entry: rust/lib.rs\n  target: shared\ndependencies:\n  runtime:\n    - std\nside_effects:\n  - none\ntimeouts:\n  default_ms: 1000\nresource_limits:\n  memory_mb: 16\nfailure_modes:\n  - id: invalid_input\n    when: invalid\nerror_codes:\n  - invalid_input\nrecovery_strategy:\n  - retry\nverification:\n  automated:\n    - cargo test\nevaluation:\n  quality_gates:\n    - stable\nacceptance_criteria:\n  - works\n{extra}"
        ),
    )
    .expect("contract should be written");
    fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
}

#[test]
fn errors_when_blocks_root_is_missing() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let missing_root = temp_dir.path().join("missing");

    let result = Registry::load_from_root(&missing_root);

    assert!(matches!(result, Err(RegistryError::MissingRoot(path)) if path == missing_root));
}

#[test]
fn errors_on_duplicate_block_ids() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    write_contract(&temp_dir, "first", "demo.echo", "Echo A");
    write_contract(&temp_dir, "second", "demo.echo", "Echo B");

    let result = Registry::load_from_root(temp_dir.path());

    assert!(matches!(result, Err(RegistryError::DuplicateBlockId(id)) if id == "demo.echo"));
}

#[test]
fn discovers_blocks_and_supports_search() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    write_contract(&temp_dir, "http", "core.http.get", "HTTP Get");
    write_contract(&temp_dir, "file", "core.fs.read_text", "Read Text");

    let registry = Registry::load_from_root(temp_dir.path()).expect("registry should load");

    assert_eq!(registry.list().len(), 2);
    let block = registry
        .get("core.http.get")
        .expect("http block should exist");
    assert!(block.implementation_path.ends_with("rust/lib.rs"));
    assert_eq!(registry.search("read").len(), 1);
    assert_eq!(registry.search("http").len(), 1);
}

#[test]
fn active_contract_warnings_when_enforcement_is_warn() {
    let _guard = env_guard().lock().expect("env lock");
    unsafe {
        std::env::set_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV, "warn");
    }

    let temp_dir = TempDir::new().expect("temp dir should be created");
    write_contract_with_extra(&temp_dir, "active", "demo.active", "Active Demo", "");
    let path = temp_dir.path().join("active").join("block.yaml");
    let source = fs::read_to_string(&path).expect("contract should be readable");
    fs::write(path, source.replace("status: candidate", "status: active"))
        .expect("contract should be updated");

    let result = Registry::load_from_root(temp_dir.path());

    unsafe {
        std::env::remove_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV);
    }

    let registry = result.expect("warn mode should load registry");
    let block = registry
        .get("demo.active")
        .expect("active block should exist");
    assert_eq!(block.contract_warnings.len(), 3);
}

#[test]
fn active_contract_errors_when_enforcement_is_error() {
    let _guard = env_guard().lock().expect("env lock");
    unsafe {
        std::env::set_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV, "error");
    }

    let temp_dir = TempDir::new().expect("temp dir should be created");
    write_contract_with_extra(&temp_dir, "active", "demo.active", "Active Demo", "");
    let path = temp_dir.path().join("active").join("block.yaml");
    let source = fs::read_to_string(&path).expect("contract should be readable");
    fs::write(path, source.replace("status: candidate", "status: active"))
        .expect("contract should be updated");

    let result = Registry::load_from_root(temp_dir.path());

    unsafe {
        std::env::remove_var(ACTIVE_REQUIRED_FIELDS_ENFORCEMENT_ENV);
    }

    assert!(matches!(result, Err(RegistryError::ParseContract { .. })));
}
