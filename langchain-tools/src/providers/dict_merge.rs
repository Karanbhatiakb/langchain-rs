//! Dict merge tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that merges multiple dictionaries.
#[derive(Debug, Clone)]
pub struct DictMergeTool;

impl DictMergeTool {
    /// Create a new `DictMergeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DictMergeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DictMergeTool {
    fn name(&self) -> &str {
        "dict_merge"
    }

    fn description(&self) -> &str {
        "Merges two or more dictionaries into a single one."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DictMergeTool is a stub");
        Ok("Result from dict_merge".into())
    }
}
