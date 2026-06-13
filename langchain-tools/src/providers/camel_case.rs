//! Camel case string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a string to camelCase.
#[derive(Debug, Clone)]
pub struct CamelCaseTool;

impl CamelCaseTool {
    /// Create a new `CamelCaseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CamelCaseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CamelCaseTool {
    fn name(&self) -> &str {
        "camel_case"
    }

    fn description(&self) -> &str {
        "Converts the input string to camelCase."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CamelCaseTool is a stub");
        Ok("Result from camel_case".into())
    }
}
