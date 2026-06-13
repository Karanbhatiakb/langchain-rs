//! Google Finance tool for financial data.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that fetches financial data from Google Finance.
#[derive(Debug)]
pub struct GoogleFinanceTool;

impl GoogleFinanceTool {
    /// Creates a new [`GoogleFinanceTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GoogleFinanceTool {
    fn name(&self) -> &str {
        "google_finance"
    }

    fn description(&self) -> &str {
        "Fetches financial data from Google Finance"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Google Finance not configured (stub)".into(),
        ))
    }
}
