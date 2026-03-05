use blocks_runtime::{BlockExecutionError, BlockRunner};
use serde_json::Value;

include!(concat!(env!("OUT_DIR"), "/generated_catalog.rs"));

#[derive(Debug, Default, Clone, Copy)]
struct CatalogBlockRunner;

pub fn default_block_runner() -> impl BlockRunner {
    CatalogBlockRunner
}

pub fn registered_block_ids() -> &'static [&'static str] {
    REGISTERED_BLOCK_IDS
}

impl BlockRunner for CatalogBlockRunner {
    fn run(&self, block_id: &str, input: &Value) -> Result<Value, BlockExecutionError> {
        match dispatch_registered_block(block_id, input) {
            Some(result) => result,
            None => Err(BlockExecutionError::new(format!(
                "no executor registered for block: {block_id}"
            ))),
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
#[path = "codegen.rs"]
mod codegen;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::Path;
    use std::thread;

    use blocks_contract::BlockContract;
    use blocks_runtime::{BlockRunner, Runtime};
    use serde_json::json;
    use tempfile::TempDir;

    use super::{codegen, default_block_runner, registered_block_ids};

    #[test]
    fn dispatches_registered_demo_echo_block() {
        let runner = default_block_runner();
        let output = runner
            .run("demo.echo", &json!({ "text": "hello" }))
            .expect("registered block should run");

        assert_eq!(output, json!({ "text": "hello" }));
    }

    #[test]
    fn returns_exact_fallback_for_unknown_blocks() {
        let runner = default_block_runner();
        let error = runner
            .run("demo.unknown", &json!({}))
            .expect_err("unknown block should fail");

        assert_eq!(
            error.to_string(),
            "no executor registered for block: demo.unknown"
        );
    }

    #[test]
    fn exposes_registered_block_ids_in_deterministic_order() {
        assert_eq!(
            registered_block_ids(),
            &[
                "core.console.write_line",
                "core.fs.read_text",
                "core.fs.write_text",
                "core.http.get",
                "core.json.transform",
                "core.llm.chat",
                "demo.echo",
            ]
        );
    }

    #[test]
    fn executes_core_http_get_through_runtime_contract_validation() {
        let contract = BlockContract::from_yaml_str(
            r#"
id: core.http.get
name: HTTP Get
version: 0.1.0
status: candidate
owner: blocks-core-team
purpose: test http get
scope:
  - call endpoint
non_goals:
  - https
inputs:
  - name: url
    description: endpoint
implementation:
  kind: rust
  entry: rust/lib.rs
  target: backend
input_schema:
  url:
    type: string
    required: true
output_schema:
  status:
    type: integer
    required: true
  body:
    type: string
    required: true
preconditions:
  - url provided
outputs:
  - name: status
    description: status code
postconditions:
  - status exists
dependencies:
  runtime:
    - std
side_effects:
  - network call
timeouts:
  default_ms: 1000
resource_limits:
  memory_mb: 16
failure_modes:
  - id: invalid_input
    when: invalid input
error_codes:
  - invalid_input
recovery_strategy:
  - retry
verification:
  automated:
    - cargo test
evaluation:
  quality_gates:
    - stable
acceptance_criteria:
  - returns status and body
"#,
        )
        .expect("contract should parse");
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let address = listener
            .local_addr()
            .expect("local addr should be available");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request should arrive");
            let mut buffer = [0_u8; 512];
            let _ = stream
                .read(&mut buffer)
                .expect("request should be readable");
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello")
                .expect("response should be written");
        });
        let runner = default_block_runner();
        let result = Runtime::new()
            .execute(
                &contract,
                &json!({ "url": format!("http://127.0.0.1:{}/", address.port()) }),
                &runner,
            )
            .expect("registered block should satisfy runtime contract validation");

        server.join().expect("server should finish");

        assert_eq!(result.output, json!({ "status": 200, "body": "hello" }));
        assert_eq!(result.record.block_id, "core.http.get");
        assert!(result.record.success);
    }

    #[test]
    fn codegen_sorts_catalog_entries_by_block_id() {
        let workspace = TempDir::new().expect("temp dir should exist");
        let manifest_path = write_test_manifest(
            workspace.path(),
            &[
                ("block-zeta", "../blocks/zeta/rust"),
                ("block-alpha", "../blocks/alpha/rust"),
            ],
        );
        write_test_block(workspace.path(), "alpha", "alpha.block", false);
        write_test_block(workspace.path(), "zeta", "zeta.block", false);

        let blocks = codegen::collect_registered_blocks(&manifest_path)
            .expect("codegen should parse blocks");

        let ids: Vec<_> = blocks.iter().map(|block| block.block_id.as_str()).collect();
        assert_eq!(ids, vec!["alpha.block", "zeta.block"]);

        let rendered = codegen::render_dispatch_glue(&blocks);
        let alpha_position = rendered
            .find("\"alpha.block\"")
            .expect("alpha block should be rendered");
        let zeta_position = rendered
            .find("\"zeta.block\"")
            .expect("zeta block should be rendered");
        assert!(alpha_position < zeta_position);
    }

    #[test]
    fn codegen_rejects_invalid_block_metadata() {
        let workspace = TempDir::new().expect("temp dir should exist");
        let manifest_path = write_test_manifest(
            workspace.path(),
            &[("block-broken", "../blocks/broken/rust")],
        );
        write_test_block(workspace.path(), "broken", "broken.block", true);

        let error = codegen::collect_registered_blocks(&manifest_path)
            .expect_err("invalid metadata should fail codegen");

        assert!(
            error
                .to_string()
                .contains("implementation.entry must not be empty"),
            "unexpected error: {error}"
        );
    }

    fn write_test_manifest(root: &Path, dependencies: &[(&str, &str)]) -> std::path::PathBuf {
        let catalog_dir = root.join("catalog");
        fs::create_dir_all(&catalog_dir).expect("catalog dir should exist");
        let manifest_path = catalog_dir.join("Cargo.toml");
        let mut source = String::from(
            "[package]\nname = \"test-catalog\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde_json = \"1\"\n",
        );
        for (dependency_name, path) in dependencies {
            source.push_str(dependency_name);
            source.push_str(" = { path = \"");
            source.push_str(path);
            source.push_str("\" }\n");
        }
        fs::write(&manifest_path, source).expect("manifest should be written");
        manifest_path
    }

    fn write_test_block(root: &Path, name: &str, block_id: &str, invalid_entry: bool) {
        let block_dir = root.join("blocks").join(name);
        let rust_dir = block_dir.join("rust");
        fs::create_dir_all(&rust_dir).expect("rust dir should exist");
        let entry = if invalid_entry { "\"\"" } else { "rust/lib.rs" };
        let block_yaml = format!(
            "id: {block_id}\nname: Test Block\nversion: 0.1.0\nstatus: candidate\nowner: blocks-core-team\npurpose: test block\nscope:\n  - test\nnon_goals:\n  - none\ninputs:\n  - name: value\n    description: value\ninput_schema:\n  value:\n    type: string\n    required: true\npreconditions:\n  - input exists\noutputs:\n  - name: value\n    description: value\noutput_schema:\n  value:\n    type: string\n    required: true\npostconditions:\n  - output exists\nimplementation:\n  kind: rust\n  entry: {entry}\n  target: shared\ndependencies:\n  runtime:\n    - std\nside_effects:\n  - none\ntimeouts:\n  default_ms: 100\nresource_limits:\n  memory_mb: 16\nfailure_modes:\n  - id: invalid_input\n    when: invalid\nerror_codes:\n  - invalid_input\nrecovery_strategy:\n  - retry\nverification:\n  automated:\n    - cargo test\nevaluation:\n  quality_gates:\n    - stable\nacceptance_criteria:\n  - works\n"
        );
        fs::write(block_dir.join("block.yaml"), block_yaml).expect("block metadata should exist");
    }
}
