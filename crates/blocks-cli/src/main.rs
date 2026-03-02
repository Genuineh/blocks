use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::ExitCode;

use blocks_composer::{AppManifest, Composer};
use blocks_registry::Registry;
use blocks_runtime::{BlockExecutionError, BlockRunner, Runtime};
use serde_json::{Value, json};

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

            let input_source = fs::read_to_string(input_path)
                .map_err(|error| format!("failed to read input file {input_path}: {error}"))?;
            let input: Value = serde_json::from_str(&input_source)
                .map_err(|error| format!("failed to parse input JSON {input_path}: {error}"))?;

            let runtime = Runtime::new();
            let runner = CliBlockRunner;
            let result = runtime
                .execute(&block.contract, &input, &runner)
                .map_err(|error| error.to_string())?;

            serde_json::to_string_pretty(&result.output)
                .map_err(|error| format!("failed to render output JSON: {error}"))
        }
        [command, subcommand, root, manifest_path, input_path]
            if command == "compose" && subcommand == "run" =>
        {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let manifest_source = fs::read_to_string(manifest_path)
                .map_err(|error| format!("failed to read manifest {manifest_path}: {error}"))?;
            let manifest = AppManifest::from_yaml_str(&manifest_source)
                .map_err(|error| format!("failed to load manifest {manifest_path}: {error}"))?;

            let input_source = fs::read_to_string(input_path)
                .map_err(|error| format!("failed to read input file {input_path}: {error}"))?;
            let input: Value = serde_json::from_str(&input_source)
                .map_err(|error| format!("failed to parse input JSON {input_path}: {error}"))?;

            let result = Composer::new()
                .execute(&manifest, &input, &registry, &CliBlockRunner)
                .map_err(|error| error.to_string())?;

            serde_json::to_string_pretty(&result.output)
                .map_err(|error| format!("failed to render output JSON: {error}"))
        }
        _ => Err(
            "usage: blocks <list|show|search> <blocks-root> [query|block-id]\n       blocks run <blocks-root> <block-id> <input-json-file>\n       blocks compose run <blocks-root> <app-yaml> <input-json-file>"
                .to_string(),
        ),
    }
}

struct CliBlockRunner;

impl BlockRunner for CliBlockRunner {
    fn run(&self, block_id: &str, input: &Value) -> Result<Value, BlockExecutionError> {
        match block_id {
            "demo.echo" => {
                let text = input
                    .get("text")
                    .cloned()
                    .unwrap_or_else(|| Value::String(String::new()));
                Ok(json!({ "text": text }))
            }
            "core.fs.read_text" => {
                let path = input
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BlockExecutionError::new("missing string field: path"))?;
                let text = fs::read_to_string(path).map_err(|error| {
                    BlockExecutionError::new(format!("failed to read file {path}: {error}"))
                })?;
                Ok(json!({ "text": text }))
            }
            "core.fs.write_text" => {
                let path = input
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BlockExecutionError::new("missing string field: path"))?;
                let text = input
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BlockExecutionError::new("missing string field: text"))?;
                fs::write(path, text).map_err(|error| {
                    BlockExecutionError::new(format!("failed to write file {path}: {error}"))
                })?;
                Ok(json!({ "path": path }))
            }
            "core.json.transform" => {
                let source = input
                    .get("source")
                    .cloned()
                    .ok_or_else(|| BlockExecutionError::new("missing object field: source"))?;
                Ok(json!({ "result": source }))
            }
            "core.http.get" => run_core_http_get(input),
            "core.llm.chat" => run_core_llm_chat(input),
            _ => Err(BlockExecutionError::new(format!(
                "no executor registered for block: {block_id}"
            ))),
        }
    }
}

fn run_core_http_get(input: &Value) -> Result<Value, BlockExecutionError> {
    let url = input
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| BlockExecutionError::new("missing string field: url"))?;

    let (host, port, path) = parse_plain_http_url(url)?;
    let mut stream = TcpStream::connect((host.as_str(), port)).map_err(|error| {
        BlockExecutionError::new(format!("failed to connect to {host}:{port}: {error}"))
    })?;

    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).map_err(|error| {
        BlockExecutionError::new(format!("failed to write request to {host}:{port}: {error}"))
    })?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).map_err(|error| {
        BlockExecutionError::new(format!(
            "failed to read response from {host}:{port}: {error}"
        ))
    })?;

    let response = String::from_utf8_lossy(&buffer);
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| BlockExecutionError::new("invalid HTTP response"))?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| BlockExecutionError::new("invalid HTTP status line"))?;

    Ok(json!({
        "status": status,
        "body": body,
    }))
}

fn run_core_llm_chat(input: &Value) -> Result<Value, BlockExecutionError> {
    let prompt = input
        .get("prompt")
        .and_then(Value::as_str)
        .ok_or_else(|| BlockExecutionError::new("missing string field: prompt"))?;

    Ok(json!({
        "text": prompt,
    }))
}

fn parse_plain_http_url(url: &str) -> Result<(String, u16, String), BlockExecutionError> {
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        BlockExecutionError::new("only plain http:// URLs are supported in the current MVP runner")
    })?;

    let (host_port, path) = match rest.split_once('/') {
        Some((host_port, path)) => (host_port, format!("/{path}")),
        None => (rest, "/".to_string()),
    };

    if host_port.is_empty() {
        return Err(BlockExecutionError::new("missing host in url"));
    }

    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|_| BlockExecutionError::new("invalid port in url"))?;
            (host.to_string(), port)
        }
        None => (host_port.to_string(), 80),
    };

    if host.is_empty() {
        return Err(BlockExecutionError::new("missing host in url"));
    }

    Ok((host, port, path))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use tempfile::TempDir;

    use super::run;

    #[test]
    fn runs_demo_echo_block_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        let block_dir = blocks_root.join("demo.echo");
        fs::create_dir_all(&block_dir).expect("block dir should be created");
        fs::write(
            block_dir.join("block.yaml"),
            r#"
id: demo.echo
name: Demo Echo
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        )
        .expect("contract should be written");

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
    fn runs_compose_pipeline_from_cli_command() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        let block_dir = blocks_root.join("demo.echo");
        fs::create_dir_all(&block_dir).expect("block dir should be created");
        fs::write(
            block_dir.join("block.yaml"),
            r#"
id: demo.echo
name: Demo Echo
input_schema:
  text:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        )
        .expect("contract should be written");

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
      - id: first
        block: demo.echo
      - id: second
        block: demo.echo
    binds:
      - from: input.text
        to: first.text
      - from: first.text
        to: second.text
"#,
        )
        .expect("manifest should be written");

        let input_path = temp_dir.path().join("input.json");
        fs::write(&input_path, r#"{ "text": "hello pipeline" }"#).expect("input should be written");

        let output = run(vec![
            "compose".to_string(),
            "run".to_string(),
            blocks_root.display().to_string(),
            manifest_path.display().to_string(),
            input_path.display().to_string(),
        ])
        .expect("compose command should succeed");

        assert!(output.contains("\"text\": \"hello pipeline\""));
    }

    #[test]
    fn runs_core_fs_write_and_read_blocks() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");

        let write_dir = blocks_root.join("core.fs.write_text");
        fs::create_dir_all(&write_dir).expect("write block dir should be created");
        fs::write(
            write_dir.join("block.yaml"),
            r#"
id: core.fs.write_text
name: Write Text File
input_schema:
  path:
    type: string
    required: true
  text:
    type: string
    required: true
output_schema:
  path:
    type: string
    required: true
"#,
        )
        .expect("write block contract should be written");

        let read_dir = blocks_root.join("core.fs.read_text");
        fs::create_dir_all(&read_dir).expect("read block dir should be created");
        fs::write(
            read_dir.join("block.yaml"),
            r#"
id: core.fs.read_text
name: Read Text File
input_schema:
  path:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        )
        .expect("read block contract should be written");

        let file_path = temp_dir.path().join("note.txt");

        let write_input = temp_dir.path().join("write.json");
        fs::write(
            &write_input,
            format!(
                r#"{{ "path": "{}", "text": "hello file" }}"#,
                file_path.display()
            ),
        )
        .expect("write input should be written");

        let write_output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "core.fs.write_text".to_string(),
            write_input.display().to_string(),
        ])
        .expect("write command should succeed");

        assert!(write_output.contains("note.txt"));

        let read_input = temp_dir.path().join("read.json");
        fs::write(
            &read_input,
            format!(r#"{{ "path": "{}" }}"#, file_path.display()),
        )
        .expect("read input should be written");

        let read_output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "core.fs.read_text".to_string(),
            read_input.display().to_string(),
        ])
        .expect("read command should succeed");

        assert!(read_output.contains("\"text\": \"hello file\""));
    }

    #[test]
    fn runs_core_json_transform_block() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        let block_dir = blocks_root.join("core.json.transform");
        fs::create_dir_all(&block_dir).expect("json block dir should be created");
        fs::write(
            block_dir.join("block.yaml"),
            r#"
id: core.json.transform
name: JSON Transform
input_schema:
  source:
    type: object
    required: true
output_schema:
  result:
    type: object
    required: true
"#,
        )
        .expect("json block contract should be written");

        let input_path = temp_dir.path().join("input.json");
        fs::write(&input_path, r#"{ "source": { "name": "blocks" } }"#)
            .expect("input should be written");

        let output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "core.json.transform".to_string(),
            input_path.display().to_string(),
        ])
        .expect("json transform command should succeed");

        assert!(output.contains("\"result\""));
        assert!(output.contains("\"name\": \"blocks\""));
    }

    #[test]
    fn runs_core_http_get_block_against_local_server() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        let block_dir = blocks_root.join("core.http.get");
        fs::create_dir_all(&block_dir).expect("http block dir should be created");
        fs::write(
            block_dir.join("block.yaml"),
            r#"
id: core.http.get
name: HTTP Get
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
        )
        .expect("http block contract should be written");

        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let address = listener.local_addr().expect("local addr should exist");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("connection should be accepted");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\nhello world",
                )
                .expect("response should be written");
        });

        let input_path = temp_dir.path().join("input.json");
        fs::write(
            &input_path,
            format!(r#"{{ "url": "http://{}/" }}"#, address),
        )
        .expect("input should be written");

        let output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "core.http.get".to_string(),
            input_path.display().to_string(),
        ])
        .expect("http get should succeed");

        server.join().expect("server thread should complete");

        assert!(output.contains("\"status\": 200"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn runs_core_llm_chat_block() {
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let blocks_root = temp_dir.path().join("blocks");
        let block_dir = blocks_root.join("core.llm.chat");
        fs::create_dir_all(&block_dir).expect("llm block dir should be created");
        fs::write(
            block_dir.join("block.yaml"),
            r#"
id: core.llm.chat
name: LLM Chat
input_schema:
  prompt:
    type: string
    required: true
output_schema:
  text:
    type: string
    required: true
"#,
        )
        .expect("llm block contract should be written");

        let input_path = temp_dir.path().join("input.json");
        fs::write(&input_path, r#"{ "prompt": "hello model" }"#).expect("input should be written");

        let output = run(vec![
            "run".to_string(),
            blocks_root.display().to_string(),
            "core.llm.chat".to_string(),
            input_path.display().to_string(),
        ])
        .expect("llm chat should succeed");

        assert!(output.contains("\"text\": \"hello model\""));
    }
}
