//! XML parse tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that parses an XML string.
#[derive(Debug, Clone)]
pub struct XmlParseTool;

impl XmlParseTool {
    /// Create a new `XmlParseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for XmlParseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for XmlParseTool {
    fn name(&self) -> &str {
        "xml_parse"
    }

    fn description(&self) -> &str {
        "Parses an XML string and returns a structured representation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("XmlParseTool is a stub");
        Ok("Result from xml_parse".into())
    }
}
