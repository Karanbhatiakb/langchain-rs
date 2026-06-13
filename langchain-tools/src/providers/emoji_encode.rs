//! Emoji encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes emoji characters in a string to their shortcode representation..
#[derive(Debug, Clone)]
pub struct EmojiEncodeTool;

impl EmojiEncodeTool {
    /// Create a new `EmojiEncodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmojiEncodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EmojiEncodeTool {
    fn name(&self) -> &str {
        "emoji_encode"
    }

    fn description(&self) -> &str {
        "Encodes emoji characters in a string to their shortcode representation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EmojiEncodeTool is a stub");
        Ok("Result from emoji_encode".into())
    }
}
