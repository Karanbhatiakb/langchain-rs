//! Steam game information tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that fetches game information from Steam.
#[derive(Debug)]
pub struct SteamTool;

impl SteamTool {
    /// Creates a new [`SteamTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SteamTool {
    fn name(&self) -> &str {
        "steam"
    }

    fn description(&self) -> &str {
        "Fetches game information from Steam"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Steam API not configured (stub)".into(),
        ))
    }
}
