//! Yahoo Finance news tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that fetches financial news from Yahoo Finance.
#[derive(Debug)]
pub struct YahooFinanceNewsTool;

impl YahooFinanceNewsTool {
    /// Creates a new [`YahooFinanceNewsTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for YahooFinanceNewsTool {
    fn name(&self) -> &str {
        "yahoo_finance_news"
    }

    fn description(&self) -> &str {
        "Fetches financial news from Yahoo Finance"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Yahoo Finance not configured (stub)".into(),
        ))
    }
}
