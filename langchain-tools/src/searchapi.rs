//! SearchApi tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches using SearchApi.
#[derive(Debug)]
pub struct SearchApiTool;

impl SearchApiTool {
    /// Creates a new [`SearchApiTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SearchApiTool {
    fn name(&self) -> &str {
        "searchapi"
    }

    fn description(&self) -> &str {
        "Searches using SearchApi"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "SearchApi not available (stub)".into(),
        ))
    }
}
