//! Dict keys tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the keys of a dictionary.
#[derive(Debug, Clone)]
pub struct DictKeysTool;

impl DictKeysTool {
    /// Create a new `DictKeysTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DictKeysTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DictKeysTool {
    fn name(&self) -> &str {
        "dict_keys"
    }

    fn description(&self) -> &str {
        "Returns the keys of a dictionary object."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DictKeysTool is a stub");
        Ok("Result from dict_keys".into())
    }
}
