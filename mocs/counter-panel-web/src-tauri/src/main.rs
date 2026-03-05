use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if env::args().skip(1).any(|arg| arg == "--headless-probe") {
        run_headless_probe();
        return;
    }

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn run_headless_probe() {
    let preview_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("preview")
        .join("index.html");

    if !preview_path.is_file() {
        eprintln!(
            "frontend host is configured, but preview is missing: {}",
            preview_path.display()
        );
        std::process::exit(1);
    }

    let rendered_path = match fs::canonicalize(&preview_path) {
        Ok(path) => path.display().to_string(),
        Err(_) => preview_path.display().to_string(),
    };

    println!("tauri host ready: {rendered_path}");
}
