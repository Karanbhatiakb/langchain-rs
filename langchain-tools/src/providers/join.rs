//! String join tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that joins multiple strings with a separator.
#[derive(Debug, Clone)]
pub struct JoinTool;

impl JoinTool {
    /// Create a new `JoinTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JoinTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for JoinTool {
    fn name(&self) -> &str {
        "join"
    }

    fn description(&self) -> &str {
        "Joins multiple strings together with a given separator."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("JoinTool is a stub");
        Ok("Result from join".into())
    }
}
