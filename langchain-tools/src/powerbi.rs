//! Power BI dataset query tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that queries Power BI datasets.
#[derive(Debug)]
pub struct PowerBITool;

impl PowerBITool {
    /// Creates a new [`PowerBITool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for PowerBITool {
    fn name(&self) -> &str {
        "powerbi"
    }

    fn description(&self) -> &str {
        "Queries Power BI datasets"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "PowerBI not configured (stub)".into(),
        ))
    }
}
