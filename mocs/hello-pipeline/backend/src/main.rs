use std::env;
use std::fs;
use std::process::ExitCode;

use serde_json::Value;

fn main() -> ExitCode {
    let input_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "mocs/hello-pipeline/input.example.json".to_string());
    let input_source = match fs::read_to_string(&input_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read input file {input_path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let input: Value = match serde_json::from_str(&input_source) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("failed to parse input JSON {input_path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let path = match input.get("path").and_then(Value::as_str) {
        Some(path) => path,
        None => {
            eprintln!("missing string field: path");
            return ExitCode::FAILURE;
        }
    };
    let text = match input.get("text").and_then(Value::as_str) {
        Some(text) => text,
        None => {
            eprintln!("missing string field: text");
            return ExitCode::FAILURE;
        }
    };
    let write_input = serde_json::json!({
        "path": path,
        "text": text,
    });
    let write_output = match block_core_fs_write_text::run(&write_input) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let written_path = match write_output.get("path").and_then(Value::as_str) {
        Some(path) => path,
        None => {
            eprintln!("write_text returned invalid output");
            return ExitCode::FAILURE;
        }
    };
    let read_input = serde_json::json!({
        "path": written_path,
    });
    let output = match block_core_fs_read_text::run(&read_input) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    match serde_json::to_string_pretty(&output) {
        Ok(rendered) => {
            println!("{rendered}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to render output JSON: {error}");
            ExitCode::FAILURE
        }
    }
}
