//! Template string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that renders a template string with variable substitutions.
#[derive(Debug, Clone)]
pub struct TemplateTool;

impl TemplateTool {
    /// Create a new `TemplateTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TemplateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for TemplateTool {
    fn name(&self) -> &str {
        "template"
    }

    fn description(&self) -> &str {
        "Renders a template string by substituting {{variables}} with provided values."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("TemplateTool is a stub");
        Ok("Result from template".into())
    }
}
