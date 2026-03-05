use std::env;
use std::process::ExitCode;

use greeting_api_service_backend::{DEFAULT_PORT, Greeting, bind_listener, serve};

fn main() -> ExitCode {
    let port = match resolve_port() {
        Ok(port) => port,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let listener = match bind_listener(port) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("failed to bind greeting-api-service on 127.0.0.1:{port}: {error}");
            return ExitCode::FAILURE;
        }
    };

    println!("greeting-api-service listening on http://127.0.0.1:{port}");

    match serve(listener, Greeting::demo(), None) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("greeting-api-service stopped with an error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn resolve_port() -> Result<u16, String> {
    match env::var("GREETING_API_PORT") {
        Ok(value) => value
            .parse::<u16>()
            .map_err(|_| format!("invalid GREETING_API_PORT value: {value}")),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_PORT),
        Err(error) => Err(format!("failed to read GREETING_API_PORT: {error}")),
    }
}
