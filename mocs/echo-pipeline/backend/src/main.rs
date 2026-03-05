use std::env;
use std::fs;
use std::process::ExitCode;

use serde_json::Value;

fn main() -> ExitCode {
    let input_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "mocs/echo-pipeline/input.example.json".to_string());
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
    let text = match input.get("text") {
        Some(text) => text.clone(),
        None => {
            eprintln!("missing field: text");
            return ExitCode::FAILURE;
        }
    };
    let first_output = match block_demo_echo::run(&serde_json::json!({ "text": text })) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let output = match block_demo_echo::run(&first_output) {
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
