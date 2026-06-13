//! MD5 hash tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the MD5 hash of a string.
#[derive(Debug, Clone)]
pub struct Md5HashTool;

impl Md5HashTool {
    /// Create a new `Md5HashTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Md5HashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for Md5HashTool {
    fn name(&self) -> &str {
        "md5_hash"
    }

    fn description(&self) -> &str {
        "Computes the MD5 hash of the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("Md5HashTool is a stub");
        Ok("Result from md5_hash".into())
    }
}
