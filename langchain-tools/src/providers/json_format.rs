//! JSON format tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that pretty-prints a JSON string.
#[derive(Debug, Clone)]
pub struct JsonFormatTool;

impl JsonFormatTool {
    /// Create a new `JsonFormatTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormatTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for JsonFormatTool {
    fn name(&self) -> &str {
        "json_format"
    }

    fn description(&self) -> &str {
        "Pretty-prints a JSON string with proper indentation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("JsonFormatTool is a stub");
        Ok("Result from json_format".into())
    }
}
