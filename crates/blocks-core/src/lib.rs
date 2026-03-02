use blocks_runtime::{BlockExecutionError, BlockRunner};
use serde_json::Value;

#[path = "../../../blocks/core.fs.read_text/rust/lib.rs"]
mod core_fs_read_text;
#[path = "../../../blocks/core.fs.write_text/rust/lib.rs"]
mod core_fs_write_text;
#[path = "../../../blocks/core.http.get/rust/lib.rs"]
mod core_http_get;
#[path = "../../../blocks/core.json.transform/rust/lib.rs"]
mod core_json_transform;
#[path = "../../../blocks/core.llm.chat/rust/lib.rs"]
mod core_llm_chat;
#[path = "../../../blocks/demo.echo/rust/lib.rs"]
mod demo_echo;

#[derive(Debug, Default, Clone, Copy)]
pub struct LibraryBlockRunner;

impl BlockRunner for LibraryBlockRunner {
    fn run(&self, block_id: &str, input: &Value) -> Result<Value, BlockExecutionError> {
        match block_id {
            "demo.echo" => demo_echo::run(input),
            "core.fs.read_text" => core_fs_read_text::run(input),
            "core.fs.write_text" => core_fs_write_text::run(input),
            "core.json.transform" => core_json_transform::run(input),
            "core.http.get" => core_http_get::run(input),
            "core.llm.chat" => core_llm_chat::run(input),
            _ => Err(BlockExecutionError::new(format!(
                "no executor registered for block: {block_id}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use blocks_runtime::BlockRunner;
    use serde_json::json;

    use super::LibraryBlockRunner;

    #[test]
    fn runs_demo_echo() {
        let output = LibraryBlockRunner
            .run("demo.echo", &json!({ "text": "hello" }))
            .expect("block should run");

        assert_eq!(output, json!({ "text": "hello" }));
    }

    #[test]
    fn runs_mock_llm_chat() {
        let output = LibraryBlockRunner
            .run("core.llm.chat", &json!({ "prompt": "hi" }))
            .expect("block should run");

        assert_eq!(output, json!({ "text": "hi" }));
    }
}
