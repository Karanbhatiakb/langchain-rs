//! Dict values tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the values of a dictionary.
#[derive(Debug, Clone)]
pub struct DictValuesTool;

impl DictValuesTool {
    /// Create a new `DictValuesTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DictValuesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DictValuesTool {
    fn name(&self) -> &str {
        "dict_values"
    }

    fn description(&self) -> &str {
        "Returns the values of a dictionary object."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DictValuesTool is a stub");
        Ok("Result from dict_values".into())
    }
}
