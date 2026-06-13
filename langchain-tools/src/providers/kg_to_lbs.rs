//! Kilograms to pounds conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts kilograms to pounds.
#[derive(Debug, Clone)]
pub struct KgToLbsTool;

impl KgToLbsTool {
    /// Create a new `KgToLbsTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for KgToLbsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for KgToLbsTool {
    fn name(&self) -> &str {
        "kg_to_lbs"
    }

    fn description(&self) -> &str {
        "Converts a weight from kilograms to pounds."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("KgToLbsTool is a stub");
        Ok("Result from kg_to_lbs".into())
    }
}
