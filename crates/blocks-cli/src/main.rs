use std::env;
use std::fs;
use std::process::ExitCode;

use blocks_composer::{AppManifest, Composer};
use blocks_core::LibraryBlockRunner;
use blocks_registry::Registry;
use blocks_runtime::Runtime;
use serde_json::Value;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    match args.as_slice() {
        [command, root] if command == "list" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            Ok(registry
                .list()
                .iter()
                .map(|block| block.contract.id.as_str())
                .collect::<Vec<_>>()
                .join("\n"))
        }
        [command, root, block_id] if command == "show" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let Some(block) = registry.get(block_id) else {
                return Err(format!("block not found: {block_id}"));
            };

            let mut lines = vec![format!("id: {}", block.contract.id)];
            if let Some(name) = &block.contract.name {
                lines.push(format!("name: {name}"));
            }
            lines.push(format!("contract: {}", block.contract_path.display()));
            lines.push(format!("implementation: {}", block.implementation_path.display()));
            if let Some(implementation) = &block.contract.implementation {
                lines.push(format!("implementation_kind: {:?}", implementation.kind));
                lines.push(format!("implementation_target: {:?}", implementation.target));
            }
            Ok(lines.join("\n"))
        }
        [command, root, query] if command == "search" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            Ok(registry
                .search(query)
                .iter()
                .map(|block| block.contract.id.as_str())
                .collect::<Vec<_>>()
                .join("\n"))
        }
        [command, root, block_id, input_path] if command == "run" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let Some(block) = registry.get(block_id) else {
                return Err(format!("block not found: {block_id}"));
            };
            let input = read_json_file(input_path)?;
            let result = Runtime::new()
                .execute(&block.contract, &input, &LibraryBlockRunner)
                .map_err(|error| error.to_string())?;

            serde_json::to_string_pretty(&result.output)
                .map_err(|error| format!("failed to render output JSON: {error}"))
        }
        [command, subcommand, root, manifest_path] if command == "compose" && subcommand == "validate" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let manifest_source = fs::read_to_string(manifest_path)
                .map_err(|error| format!("failed to read manifest {manifest_path}: {error}"))?;
            let manifest = AppManifest::from_yaml_str(&manifest_source)
                .map_err(|error| format!("failed to load manifest {manifest_path}: {error}"))?;
            let plan = Composer::new()
                .plan(&manifest, &registry)
                .map_err(|error| error.to_string())?;

            Ok(format!(
                "valid: {} ({}) flow={} steps={}",
                plan.app_name,
                manifest_path,
                plan.flow_id,
                plan.steps.len()
            ))
        }
        _ => Err(
            "usage: blocks <list|show|search> <blocks-root> [query|block-id]\n       blocks run <blocks-root> <block-id> <input-json-file>\n       blocks compose validate <blocks-root> <app-yaml>"
                .to_string(),
        ),
    }
}

fn read_json_file(path: &str) -> Result<Value, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read input file {path}: {error}"))?;

    serde_json::from_str(&source)
        .map_err(|error| format!("failed to parse input JSON {path}: {error}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use tempfile::TempDir;

    use super::run;

    fn write_block(root: &std::path::Path, dir_name: &str, id: &str, body: &str) {
        let block_dir = root.join(dir_name);
        let rust_dir = block_dir.join("rust");
        fs::create_dir_all(&rust_dir).expect("block dir should be created");
        fs::write(block_dir.join("block.yaml"), body).expect("contract should be written");
        fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
        let _ = id;
    }

    #[test]
    fn runs_demo_echo_block_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let input_path = temp_dir.path().join("input.json");
        fs::write(&input_path, r#"{ "text": "hello" }"#).expect("input should be written");

        let output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "demo.echo".to_string(),
            input_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("\"text\": \"hello\""));
    }

    #[test]
    fn validates_compose_manifest_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        );

        let manifest_path = temp_dir.path().join("app.yaml");
        fs::write(
            &manifest_path,
            r#"
name: echo-pipeline
entry: main
input_schema:
  text:
    type: string
    required: true
flows:
  - id: main
    steps:
      - id: echo
        block: demo.echo
    binds:
      - from: input.text
        to: echo.text
"#,
        )
        .expect("manifest should be written");

        let output = run(vec![
            "compose".to_string(),
            "validate".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("valid: echo-pipeline"));
        assert!(output.contains("steps=1"));
    }

    #[test]
    fn shows_resolved_implementation_path() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "demo.echo",
            "demo.echo",
            r#"
id: demo.echo
name: Demo Echo
implementation:
  kind: rust
  entry: rust/lib.rs
  target: shared
"#,
        );

        let output = run(vec![
            "show".to_string(),
            blocks_root.display().to_string(),
            "demo.echo".to_string(),
        ])
        .expect("command should succeed");

        assert!(output.contains("implementation:"));
        assert!(output.contains("rust/lib.rs"));
    }

    #[test]
    fn runs_core_http_get_against_local_server() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        write_block(
            &blocks_root,
            "core.http.get",
            "core.http.get",
            r#"
id: core.http.get
name: HTTP Get
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
"#,
        );

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

        let input_path = temp_dir.path().join("input.json");
        fs::write(
            &input_path,
            format!(r#"{{ "url": "http://127.0.0.1:{}/" }}"#, address.port()),
        )
        .expect("input should be written");

        let output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "core.http.get".to_string(),
            input_path.display().to_string(),
        ])
        .expect("command should succeed");

        server.join().expect("server should finish");

        assert!(output.contains("\"status\": 200"));
        assert!(output.contains("\"body\": \"hello\""));
    }
}
