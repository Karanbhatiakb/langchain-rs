//! Country name tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that looks up the country name from an ISO country code..
#[derive(Debug, Clone)]
pub struct CountryNameTool;

impl CountryNameTool {
    /// Create a new `CountryNameTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CountryNameTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CountryNameTool {
    fn name(&self) -> &str {
        "country_name"
    }

    fn description(&self) -> &str {
        "Looks up the country name from an ISO country code."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CountryNameTool is a stub");
        Ok("Result from country_name".into())
    }
}
