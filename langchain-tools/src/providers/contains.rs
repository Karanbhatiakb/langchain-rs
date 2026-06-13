//! String contains check tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks if a string contains a substring.
#[derive(Debug, Clone)]
pub struct ContainsTool;

impl ContainsTool {
    /// Create a new `ContainsTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContainsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ContainsTool {
    fn name(&self) -> &str {
        "contains"
    }

    fn description(&self) -> &str {
        "Checks whether the input string contains a given substring."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ContainsTool is a stub");
        Ok("Result from contains".into())
    }
}
