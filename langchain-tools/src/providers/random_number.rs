//! Random number generator tool implementation.
//!
//! Provides a `RandomNumberTool` that generates random numbers within a given
//! range. Gated behind the `random_number` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for generating random numbers.
#[derive(Debug, Clone)]
pub struct RandomNumberTool;

impl RandomNumberTool {
    /// Create a new `RandomNumberTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RandomNumberTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for RandomNumberTool {
    fn name(&self) -> &str {
        "random_number"
    }

    fn description(&self) -> &str {
        "Generates a random number. Input can specify a range (e.g. '1-100'); \
         returns a random integer within that range."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("RandomNumberTool is a stub");
        Ok("42".into())
    }
}
