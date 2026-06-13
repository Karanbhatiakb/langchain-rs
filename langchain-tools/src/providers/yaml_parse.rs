//! YAML parse tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that parses a YAML string.
#[derive(Debug, Clone)]
pub struct YamlParseTool;

impl YamlParseTool {
    /// Create a new `YamlParseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for YamlParseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for YamlParseTool {
    fn name(&self) -> &str {
        "yaml_parse"
    }

    fn description(&self) -> &str {
        "Parses a YAML string and returns a structured representation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("YamlParseTool is a stub");
        Ok("Result from yaml_parse".into())
    }
}
