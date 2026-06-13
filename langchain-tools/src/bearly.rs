//! Bearly AI data extraction tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that uses Bearly AI for data extraction.
#[derive(Debug)]
pub struct BearlyTool;

impl BearlyTool {
    /// Creates a new [`BearlyTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for BearlyTool {
    fn name(&self) -> &str {
        "bearly"
    }

    fn description(&self) -> &str {
        "Uses Bearly AI for data extraction"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Bearly AI not configured (stub)".into(),
        ))
    }
}
