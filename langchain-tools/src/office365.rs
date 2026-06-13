//! Microsoft Office 365 integration tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that interacts with Microsoft Office 365 services.
#[derive(Debug)]
pub struct Office365Tool;

impl Office365Tool {
    /// Creates a new [`Office365Tool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for Office365Tool {
    fn name(&self) -> &str {
        "office365"
    }

    fn description(&self) -> &str {
        "Interacts with Microsoft Office 365 services"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Office 365 API not configured (stub)".into(),
        ))
    }
}
