//! Currency code tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that looks up the ISO currency code from a currency name..
#[derive(Debug, Clone)]
pub struct CurrencyCodeTool;

impl CurrencyCodeTool {
    /// Create a new `CurrencyCodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CurrencyCodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CurrencyCodeTool {
    fn name(&self) -> &str {
        "currency_code"
    }

    fn description(&self) -> &str {
        "Looks up the ISO currency code from a currency name."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CurrencyCodeTool is a stub");
        Ok("Result from currency_code".into())
    }
}
