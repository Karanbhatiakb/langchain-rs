//! Azure Cognitive Services tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that uses Azure Cognitive Services for text analysis.
#[derive(Debug)]
pub struct AzureCognitiveServicesTool;

impl AzureCognitiveServicesTool {
    /// Creates a new [`AzureCognitiveServicesTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for AzureCognitiveServicesTool {
    fn name(&self) -> &str {
        "azure_cognitive_services"
    }

    fn description(&self) -> &str {
        "Uses Azure Cognitive Services for text analysis"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Azure Cognitive Services not configured (stub)".into(),
        ))
    }
}
