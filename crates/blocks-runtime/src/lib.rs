use blocks_contract::{BlockContract, ValidationIssue};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub output: Value,
    pub record: ExecutionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecord {
    pub block_id: String,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct BlockExecutionError {
    message: String,
}

impl BlockExecutionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub trait BlockRunner {
    fn run(&self, block_id: &str, input: &Value) -> Result<Value, BlockExecutionError>;
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("input validation failed")]
    InputValidationFailed { issues: Vec<ValidationIssue> },
    #[error("block execution failed: {source}")]
    ExecutionFailed {
        #[source]
        source: BlockExecutionError,
    },
    #[error("output validation failed")]
    OutputValidationFailed { issues: Vec<ValidationIssue> },
}

#[derive(Debug, Default)]
pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        contract: &BlockContract,
        input: &Value,
        runner: &impl BlockRunner,
    ) -> Result<ExecutionResult, RuntimeError> {
        contract
            .validate_input(input)
            .map_err(|issues| RuntimeError::InputValidationFailed { issues })?;

        let output = runner
            .run(&contract.id, input)
            .map_err(|source| RuntimeError::ExecutionFailed { source })?;

        contract
            .validate_output(&output)
            .map_err(|issues| RuntimeError::OutputValidationFailed { issues })?;

        Ok(ExecutionResult {
            output,
            record: ExecutionRecord {
                block_id: contract.id.clone(),
                success: true,
            },
        })
    }
}
