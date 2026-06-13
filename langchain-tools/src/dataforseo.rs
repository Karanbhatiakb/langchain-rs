//! DataForSEO tool for SEO data.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that fetches SEO data from DataForSEO.
#[derive(Debug)]
pub struct DataForSeoTool;

impl DataForSeoTool {
    /// Creates a new [`DataForSeoTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for DataForSeoTool {
    fn name(&self) -> &str {
        "dataforseo"
    }

    fn description(&self) -> &str {
        "Fetches SEO data from DataForSEO"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "DataForSEO not configured (stub)".into(),
        ))
    }
}
