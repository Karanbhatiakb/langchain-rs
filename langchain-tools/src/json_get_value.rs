//! Tool for extracting a value from JSON using a key path.
//!
//! The [`JSONGetValueTool`] accepts an input string containing a JSON blob
//! followed by a newline and a dot-separated key path (e.g.
//! `{"a":{"b":1}}\na.b`). It parses the JSON, traverses the path, and
//! returns the value at that location as a string.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

/// Tool that extracts a value from a JSON object by following a
/// dot-separated key path.
///
/// # Input format
///
/// ```text
/// <JSON string>
/// <dot-separated key path>
/// ```
///
/// # Stub
///
/// This is a stub implementation. Production use should replace the body of
/// [`invoke`](JSONGetValueTool::invoke) with real JSON parsing and path
/// traversal.
#[derive(Debug, Clone)]
pub struct JSONGetValueTool;

impl JSONGetValueTool {
    /// Creates a new [`JSONGetValueTool`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for JSONGetValueTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for JSONGetValueTool {
    fn name(&self) -> &str {
        "json_get_value"
    }

    fn description(&self) -> &str {
        "Extracts a value from JSON using a dot-separated key path. \
         Input should be JSON followed by a newline and the key path, \
         e.g. `{\"a\":{\"b\":1}}\\na.b` returns `1`."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return Err(ChainError::ToolError(
                "Expected JSON string and key path separated by a newline".into(),
            ));
        }
        let json_str = parts[0].trim();
        let key_path = parts[1].trim();

        if json_str.is_empty() || key_path.is_empty() {
            return Err(ChainError::ToolError(
                "JSON string and key path must not be empty".into(),
            ));
        }

        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;

        let result = key_path
            .split('.')
            .try_fold(value, |current, key| match current.get(key) {
                Some(sub) => Ok(sub.clone()),
                None => Err(ChainError::ToolError(format!(
                    "Key '{}' not found in path '{}'",
                    key, key_path
                ))),
            })?;

        Ok(match result {
            Value::String(s) => s,
            other => serde_json::to_string(&other)
                .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?,
        })
    }
}
