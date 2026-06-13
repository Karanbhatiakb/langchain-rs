//! Golden knowledge graph query tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that queries the Golden knowledge graph.
#[derive(Debug)]
pub struct GoldenQueryTool;

impl GoldenQueryTool {
    /// Creates a new [`GoldenQueryTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GoldenQueryTool {
    fn name(&self) -> &str {
        "golden_query"
    }

    fn description(&self) -> &str {
        "Queries the Golden knowledge graph"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Golden API not configured (stub)".into(),
        ))
    }
}
