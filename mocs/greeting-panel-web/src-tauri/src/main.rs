use std::env;
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
    let required_paths = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("preview")
            .join("index.html"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("preview")
            .join("main.js"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("preview")
            .join("greeting_panel.js"),
    ];

    for path in &required_paths {
        if !path.is_file() {
            eprintln!("frontend host is configured, but asset is missing: {}", path.display());
            std::process::exit(1);
        }
    }

    println!("tauri host ready: {}", required_paths[0].display());
}
