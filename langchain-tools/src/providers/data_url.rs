//! Data URL tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that generates a data URL from binary input..
#[derive(Debug, Clone)]
pub struct DataUrlTool;

impl DataUrlTool {
    /// Create a new `DataUrlTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DataUrlTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DataUrlTool {
    fn name(&self) -> &str {
        "data_url"
    }

    fn description(&self) -> &str {
        "Generates a data URL from binary input."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DataUrlTool is a stub");
        Ok("Result from data_url".into())
    }
}
