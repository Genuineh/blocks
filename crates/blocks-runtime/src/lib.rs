#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecord {
    pub block_id: String,
    pub success: bool,
}

#[derive(Debug, Default)]
pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self
    }
}
