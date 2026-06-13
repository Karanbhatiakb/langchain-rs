//! OpenAPI spec-based request tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that makes requests based on an OpenAPI spec.
#[derive(Debug)]
pub struct OpenAPITool;

impl OpenAPITool {
    /// Creates a new [`OpenAPITool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for OpenAPITool {
    fn name(&self) -> &str {
        "openapi"
    }

    fn description(&self) -> &str {
        "Makes requests based on an OpenAPI spec"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "OpenAPI tool not configured (stub)".into(),
        ))
    }
}
