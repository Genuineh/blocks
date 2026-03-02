use std::collections::BTreeMap;
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
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    let [blocks_root, manifest_path, input_path] = args.as_slice() else {
        return Err(
            "usage: hello-pipeline-backend <blocks-root> <app-yaml> <input-json-file>".to_string(),
        );
    };

    let registry = Registry::load_from_root(blocks_root).map_err(|error| error.to_string())?;
    let manifest = load_manifest(manifest_path)?;
    let input = load_input(input_path)?;
    let plan = Composer::new()
        .plan(&manifest, &registry)
        .map_err(|error| error.to_string())?;
    let runtime = Runtime::new();
    let mut step_outputs = BTreeMap::new();

    for step in &plan.steps {
        let step_input = step
            .build_input(&input, &step_outputs)
            .map_err(|error| error.to_string())?;
        let block = registry
            .get(&step.block)
            .ok_or_else(|| format!("block not found during execution: {}", step.block))?;
        let result = runtime
            .execute(
                &block.contract,
                &Value::Object(step_input),
                &LibraryBlockRunner,
            )
            .map_err(|error| error.to_string())?;

        step_outputs.insert(step.id.clone(), result.output);
    }

    let output = step_outputs
        .remove(&plan.last_step_id)
        .ok_or_else(|| format!("missing output for final step: {}", plan.last_step_id))?;

    serde_json::to_string_pretty(&output)
        .map_err(|error| format!("failed to render output JSON: {error}"))
}

fn load_manifest(path: &str) -> Result<AppManifest, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read manifest {path}: {error}"))?;

    AppManifest::from_yaml_str(&source)
        .map_err(|error| format!("failed to load manifest {path}: {error}"))
}

fn load_input(path: &str) -> Result<Value, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read input file {path}: {error}"))?;

    serde_json::from_str(&source)
        .map_err(|error| format!("failed to parse input JSON {path}: {error}"))
}
