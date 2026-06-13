//! EdenAI multi-provider AI services tool (translation, OCR, etc.).

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that uses EdenAI for multi-provider AI services (translation, OCR, etc.).
#[derive(Debug)]
pub struct EdenAIMultiTool;

impl EdenAIMultiTool {
    /// Creates a new [`EdenAIMultiTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for EdenAIMultiTool {
    fn name(&self) -> &str {
        "edenai_multi"
    }

    fn description(&self) -> &str {
        "Uses EdenAI for multi-provider AI services (translation, OCR, etc.)"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "EdenAI multi-provider not configured (stub)".into(),
        ))
    }
}
