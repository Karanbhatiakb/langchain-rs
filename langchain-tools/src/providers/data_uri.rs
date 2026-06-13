//! Data URI encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes input as a data URI string..
#[derive(Debug, Clone)]
pub struct DataUriTool;

impl DataUriTool {
    /// Create a new `DataUriTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DataUriTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DataUriTool {
    fn name(&self) -> &str {
        "data_uri"
    }

    fn description(&self) -> &str {
        "Encodes input as a data URI string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DataUriTool is a stub");
        Ok("Result from data_uri".into())
    }
}
