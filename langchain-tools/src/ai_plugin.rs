//! AI Plugin tool for loading and invoking remote API specs.
//!
//! AI Plugins (formerly known as ChatGPT Plugins) provide a standardised
//! manifest that describes a remote API. This tool loads a plugin manifest
//! and exposes its endpoints as callable actions.

use async_trait::async_trait;
use super::traits::{BaseTool, ToolResult};

/// Tool that loads an AI Plugin (ChatGPT Plugin) manifest and invokes its
/// endpoints.
///
/// Input should be a URL to the plugin manifest (`/.well-known/ai-plugin.json`)
/// or a specific endpoint call in the format `plugin_url | endpoint | params`.
///
/// # Stub
///
/// This is a stub implementation. Wire up an HTTP client and manifest parser
/// to enable live plugin interactions.
#[derive(Debug)]
pub struct AIPluginTool;

impl AIPluginTool {
    /// Creates a new [`AIPluginTool`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for AIPluginTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AIPluginTool {
    fn name(&self) -> &str {
        "ai_plugin"
    }

    fn description(&self) -> &str {
        "Loads an AI Plugin manifest and invokes its API endpoints"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from ai_plugin (stub)".to_string())
    }
}
