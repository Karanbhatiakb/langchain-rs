//! IFTTT applet trigger tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that triggers IFTTT applets.
#[derive(Debug)]
pub struct IFTTTTool;

impl IFTTTTool {
    /// Creates a new [`IFTTTTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for IFTTTTool {
    fn name(&self) -> &str {
        "ifttt"
    }

    fn description(&self) -> &str {
        "Triggers IFTTT applets"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "IFTTT not configured (stub)".into(),
        ))
    }
}
