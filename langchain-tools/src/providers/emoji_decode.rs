//! Emoji decode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that decodes emoji shortcodes in a string to actual emoji characters..
#[derive(Debug, Clone)]
pub struct EmojiDecodeTool;

impl EmojiDecodeTool {
    /// Create a new `EmojiDecodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmojiDecodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EmojiDecodeTool {
    fn name(&self) -> &str {
        "emoji_decode"
    }

    fn description(&self) -> &str {
        "Decodes emoji shortcodes in a string to actual emoji characters."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EmojiDecodeTool is a stub");
        Ok("Result from emoji_decode".into())
    }
}
