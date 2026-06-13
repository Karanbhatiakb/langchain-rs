//! JSON query tool implementation.
//!
//! Provides a `JsonQueryTool` that queries a JSON file or value.
//! Gated behind the `json_query` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for querying JSON data using a JSONPath-like expression.
#[derive(Debug, Clone)]
pub struct JsonQueryTool;

impl JsonQueryTool {
    /// Create a new `JsonQueryTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for JsonQueryTool {
    fn name(&self) -> &str {
        "json_query"
    }

    fn description(&self) -> &str {
        "Query a JSON value using a path expression. \
         Input should be '<json_string>\\n<path>'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("JsonQueryTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
