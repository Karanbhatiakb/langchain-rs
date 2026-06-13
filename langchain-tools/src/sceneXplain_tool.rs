//! SceneXplain image description tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that describes and explains images using SceneXplain.
#[derive(Debug)]
pub struct SceneXplainTool;

impl SceneXplainTool {
    /// Creates a new [`SceneXplainTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SceneXplainTool {
    fn name(&self) -> &str {
        "scene_xplain"
    }

    fn description(&self) -> &str {
        "Describes and explains images using SceneXplain"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "SceneXplain API not configured (stub)".into(),
        ))
    }
}
