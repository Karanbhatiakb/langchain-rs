//! ROT13 cipher tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that applies the ROT13 cipher to a string.
#[derive(Debug, Clone)]
pub struct Rot13Tool;

impl Rot13Tool {
    /// Create a new `Rot13Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Rot13Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for Rot13Tool {
    fn name(&self) -> &str {
        "rot13"
    }

    fn description(&self) -> &str {
        "Applies the ROT13 cipher to the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("Rot13Tool is a stub");
        Ok("Result from rot13".into())
    }
}
