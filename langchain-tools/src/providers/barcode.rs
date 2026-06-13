//! Barcode generator tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that generates a barcode string from input data..
#[derive(Debug, Clone)]
pub struct BarcodeTool;

impl BarcodeTool {
    /// Create a new `BarcodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BarcodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BarcodeTool {
    fn name(&self) -> &str {
        "barcode"
    }

    fn description(&self) -> &str {
        "Generates a barcode string from input data."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BarcodeTool is a stub");
        Ok("Result from barcode".into())
    }
}
