//! EdenAI multi-provider AI services tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that uses EdenAI for multi-provider AI services.
#[derive(Debug)]
pub struct EdenAITool;

impl EdenAITool {
    /// Creates a new [`EdenAITool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for EdenAITool {
    fn name(&self) -> &str {
        "edenai"
    }

    fn description(&self) -> &str {
        "Uses EdenAI for multi-provider AI services"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "EdenAI not configured (stub)".into(),
        ))
    }
}
