//! LLM Math tool implementation.
//!
//! Provides a `LLMMathTool` that evaluates mathematical expressions. Gated
//! behind the `llm_math` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for performing mathematical computations.
#[derive(Debug, Clone)]
pub struct LLMMathTool;

impl LLMMathTool {
    /// Create a new `LLMMathTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for LLMMathTool {
    fn name(&self) -> &str {
        "llm_math"
    }

    fn description(&self) -> &str {
        "Evaluate a mathematical expression and return the result. Supports \
         arithmetic, exponents, trigonometry, and basic functions."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("LLMMathTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
