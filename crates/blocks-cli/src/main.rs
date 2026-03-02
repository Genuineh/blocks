use std::env;
use std::process::ExitCode;

use blocks_registry::Registry;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [command, root] if command == "list" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            for block in registry.list() {
                println!("{}", block.contract.id);
            }
            Ok(())
        }
        [command, root, block_id] if command == "show" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            let Some(block) = registry.get(block_id) else {
                return Err(format!("block not found: {block_id}"));
            };

            println!("id: {}", block.contract.id);
            if let Some(name) = &block.contract.name {
                println!("name: {name}");
            }
            println!("contract: {}", block.contract_path.display());
            Ok(())
        }
        [command, root, query] if command == "search" => {
            let registry = Registry::load_from_root(root).map_err(|error| error.to_string())?;
            for block in registry.search(query) {
                println!("{}", block.contract.id);
            }
            Ok(())
        }
        _ => Err("usage: blocks <list|show|search> <blocks-root> [query|block-id]".to_string()),
    }
}
