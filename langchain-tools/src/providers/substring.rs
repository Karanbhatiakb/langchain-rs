//! Substring extraction tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that extracts a substring from the input.
#[derive(Debug, Clone)]
pub struct SubstringTool;

impl SubstringTool {
    /// Create a new `SubstringTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SubstringTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SubstringTool {
    fn name(&self) -> &str {
        "substring"
    }

    fn description(&self) -> &str {
        "Extracts a substring from the input given start and end indices."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SubstringTool is a stub");
        Ok("Result from substring".into())
    }
}
