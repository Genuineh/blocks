use std::process::ExitCode;

fn main() -> ExitCode {
    let input = serde_json::json!({
        "text": hello_message_lib::hello_message(),
    });

    match block_core_console_write_line::run(&input) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
