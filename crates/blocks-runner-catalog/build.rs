#[path = "src/codegen.rs"]
mod codegen;

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo should set CARGO_MANIFEST_DIR"));
    let manifest_path = manifest_dir.join("Cargo.toml");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Cargo should set OUT_DIR"));
    let output_path = out_dir.join("generated_catalog.rs");

    println!("cargo:rerun-if-changed={}", manifest_path.display());

    let metadata_paths = codegen::write_generated_catalog(&manifest_path, &output_path)
        .unwrap_or_else(|error| panic!("failed to generate runner catalog glue: {error}"));

    for metadata_path in metadata_paths {
        println!("cargo:rerun-if-changed={}", metadata_path.display());
    }
}
