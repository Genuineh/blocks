mod block;
mod moc;

use blocks_registry::Registry;
use blocks_runner_catalog::default_block_runner;
use blocks_runtime::Runtime;

use crate::app::{read_json_file, resolve_diagnostics_root};
use crate::render::USAGE;

pub fn dispatch(args: Vec<String>) -> Result<String, String> {
    match args.as_slice() {
        [command, root] if command == "list" => list_command(root),
        [command, root, block_id] if command == "show" => show_command(root, block_id),
        [command, root, query] if command == "search" => search_command(root, query),
        [command, root, block_id, input_path] if command == "run" => {
            run_command(root, block_id, input_path)
        }
        [command, subcommand, root, block_id] if command == "block" && subcommand == "diagnose" => {
            block::diagnose_command(root, block_id, &[])
        }
        [command, subcommand, root, block_id, rest @ ..]
            if command == "block" && subcommand == "diagnose" =>
        {
            block::diagnose_command(root, block_id, rest)
        }
        [command, subcommand, root, manifest_path]
            if command == "moc" && subcommand == "validate" =>
        {
            moc::validate_command(root, manifest_path)
        }
        [command, subcommand, root, manifest_path] if command == "moc" && subcommand == "run" => {
            moc::run_command(root, manifest_path, None)
        }
        [command, subcommand, root, manifest_path, input_path]
            if command == "moc" && subcommand == "run" =>
        {
            moc::run_command(root, manifest_path, Some(input_path))
        }
        [command, subcommand, root, manifest_path]
            if command == "moc" && subcommand == "verify" =>
        {
            moc::verify_command(root, manifest_path, None)
        }
        [command, subcommand, root, manifest_path, input_path]
            if command == "moc" && subcommand == "verify" =>
        {
            moc::verify_command(root, manifest_path, Some(input_path))
        }
        [command, subcommand, root, manifest_path] if command == "moc" && subcommand == "dev" => {
            moc::dev_command(root, manifest_path)
        }
        [command, subcommand, root, manifest_path]
            if command == "moc" && subcommand == "diagnose" =>
        {
            moc::diagnose_command(root, manifest_path, &[])
        }
        [command, subcommand, root, manifest_path, rest @ ..]
            if command == "moc" && subcommand == "diagnose" =>
        {
            moc::diagnose_command(root, manifest_path, rest)
        }
        _ => Err(USAGE.to_string()),
    }
}

fn list_command(root: &str) -> Result<String, String> {
    let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
    Ok(registry
        .list()
        .iter()
        .map(|block| block.contract.id.as_str())
        .collect::<Vec<_>>()
        .join("\n"))
}

fn show_command(root: &str, block_id: &str) -> Result<String, String> {
    let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
    let Some(block) = registry.get(block_id) else {
        return Err(format!("block not found: {block_id}"));
    };

    let mut lines = vec![format!("id: {}", block.contract.id)];
    if let Some(name) = &block.contract.name {
        lines.push(format!("name: {name}"));
    }
    lines.push(format!("contract: {}", block.contract_path.display()));
    lines.push(format!(
        "implementation: {}",
        block.implementation_path.display()
    ));
    if let Some(implementation) = &block.contract.implementation {
        lines.push(format!("implementation_kind: {:?}", implementation.kind));
        lines.push(format!(
            "implementation_target: {:?}",
            implementation.target
        ));
    }
    Ok(lines.join("\n"))
}

fn search_command(root: &str, query: &str) -> Result<String, String> {
    let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
    Ok(registry
        .search(query)
        .iter()
        .map(|block| block.contract.id.as_str())
        .collect::<Vec<_>>()
        .join("\n"))
}

fn run_command(root: &str, block_id: &str, input_path: &str) -> Result<String, String> {
    let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
    let Some(block) = registry.get(block_id) else {
        return Err(format!("block not found: {block_id}"));
    };
    let input = read_json_file(input_path)?;
    let runner = default_block_runner();
    let runtime = Runtime::with_diagnostics_root(resolve_diagnostics_root(root));
    let result = runtime
        .execute(&block.contract, &input, &runner)
        .map_err(|error| error.to_string())?;

    serde_json::to_string_pretty(&result.output)
        .map_err(|error| format!("failed to render output JSON: {error}"))
}
