//! Currency convert tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts an amount from one currency to another..
#[derive(Debug, Clone)]
pub struct CurrencyConvertTool;

impl CurrencyConvertTool {
    /// Create a new `CurrencyConvertTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CurrencyConvertTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CurrencyConvertTool {
    fn name(&self) -> &str {
        "currency_convert"
    }

    fn description(&self) -> &str {
        "Converts an amount from one currency to another."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CurrencyConvertTool is a stub");
        Ok("Result from currency_convert".into())
    }
}
