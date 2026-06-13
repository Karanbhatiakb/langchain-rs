//! Tool for listing keys at a given path within a JSON value.
//!
//! The [`JSONListKeysTool`] accepts an input string containing a JSON blob
//! followed by a newline and an optional dot-separated key path. It parses
//! the JSON, navigates to the specified path, and returns the list of keys
//! present at that level.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

/// Tool that lists the keys of a JSON object at a given dot-separated path.
///
/// # Input format
///
/// ```text
/// <JSON string>
/// <dot-separated key path (optional)>
/// ```
///
/// If the key path is omitted or empty the tool lists the keys of the root
/// object.
///
/// # Stub
///
/// This is a stub implementation. Production use should replace the body of
/// [`invoke`](JSONListKeysTool::invoke) with real JSON parsing and path
/// traversal.
#[derive(Debug, Clone)]
pub struct JSONListKeysTool;

impl JSONListKeysTool {
    /// Creates a new [`JSONListKeysTool`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for JSONListKeysTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for JSONListKeysTool {
    fn name(&self) -> &str {
        "json_list_keys"
    }

    fn description(&self) -> &str {
        "Lists the keys of a JSON object at a dot-separated key path. \
         Input should be JSON optionally followed by a newline and a path. \
         Returns a comma-separated list of keys."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        let json_str = parts[0].trim();
        let key_path = if parts.len() > 1 {
            parts[1].trim()
        } else {
            ""
        };

        if json_str.is_empty() {
            return Err(ChainError::ToolError("JSON string must not be empty".into()));
        }

        let mut value: Value = serde_json::from_str(json_str)
            .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;

        if !key_path.is_empty() {
            value = key_path.split('.').try_fold(value, |current, key| {
                match current.get(key) {
                    Some(sub) => Ok(sub.clone()),
                    None => Err(ChainError::ToolError(format!(
                        "Key '{}' not found in path '{}'",
                        key, key_path
                    ))),
                }
            })?;
        }

        match value {
            Value::Object(map) => {
                let keys: Vec<&String> = map.keys().collect();
                if keys.is_empty() {
                    Ok("(empty object)".to_string())
                } else {
                    Ok(keys.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
                }
            }
            _ => Err(ChainError::ToolError(
                "The value at the specified path is not an object".into(),
            )),
        }
    }
}
