//! Steamship AI image generation tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that generates images using Steamship AI.
#[derive(Debug)]
pub struct SteamshipImageGenerationTool;

impl SteamshipImageGenerationTool {
    /// Creates a new [`SteamshipImageGenerationTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SteamshipImageGenerationTool {
    fn name(&self) -> &str {
        "steamship_image_generation"
    }

    fn description(&self) -> &str {
        "Generates images using Steamship AI"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Steamship API not configured (stub)".into(),
        ))
    }
}
