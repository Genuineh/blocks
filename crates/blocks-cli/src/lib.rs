pub mod app;
pub mod commands;
pub mod render;

pub fn run(args: Vec<String>) -> Result<String, String> {
    commands::dispatch(args)
}
