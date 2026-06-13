//! Emoji remove tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that removes all emoji characters from a string..
#[derive(Debug, Clone)]
pub struct EmojiRemoveTool;

impl EmojiRemoveTool {
    /// Create a new `EmojiRemoveTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmojiRemoveTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EmojiRemoveTool {
    fn name(&self) -> &str {
        "emoji_remove"
    }

    fn description(&self) -> &str {
        "Removes all emoji characters from a string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EmojiRemoveTool is a stub");
        Ok("Result from emoji_remove".into())
    }
}
