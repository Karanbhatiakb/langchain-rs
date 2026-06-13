//! Kebab case string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a string to kebab-case.
#[derive(Debug, Clone)]
pub struct KebabCaseTool;

impl KebabCaseTool {
    /// Create a new `KebabCaseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for KebabCaseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for KebabCaseTool {
    fn name(&self) -> &str {
        "kebab_case"
    }

    fn description(&self) -> &str {
        "Converts the input string to kebab-case."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("KebabCaseTool is a stub");
        Ok("Result from kebab_case".into())
    }
}
