//! E2B sandbox code execution tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that runs code in an E2B sandbox.
#[derive(Debug)]
pub struct E2BTool;

impl E2BTool {
    /// Creates a new [`E2BTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for E2BTool {
    fn name(&self) -> &str {
        "e2b"
    }

    fn description(&self) -> &str {
        "Runs code in an E2B sandbox"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "E2B sandbox not configured (stub)".into(),
        ))
    }
}
