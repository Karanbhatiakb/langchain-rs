//! Country code tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that looks up the ISO country code from a country name..
#[derive(Debug, Clone)]
pub struct CountryCodeTool;

impl CountryCodeTool {
    /// Create a new `CountryCodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CountryCodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CountryCodeTool {
    fn name(&self) -> &str {
        "country_code"
    }

    fn description(&self) -> &str {
        "Looks up the ISO country code from a country name."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CountryCodeTool is a stub");
        Ok("Result from country_code".into())
    }
}
