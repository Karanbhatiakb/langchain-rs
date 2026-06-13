//! ElevenLabs text-to-speech tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that generates speech using the ElevenLabs API.
#[derive(Debug)]
pub struct ElevenLabsTool;

impl ElevenLabsTool {
    /// Creates a new [`ElevenLabsTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for ElevenLabsTool {
    fn name(&self) -> &str {
        "eleven_labs"
    }

    fn description(&self) -> &str {
        "Generates speech using ElevenLabs API"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "ElevenLabs API not configured (stub)".into(),
        ))
    }
}
