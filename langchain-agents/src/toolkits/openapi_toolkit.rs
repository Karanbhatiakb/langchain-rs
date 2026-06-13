use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct APIRequestTool;

#[async_trait]
impl BaseTool for APIRequestTool {
    fn name(&self) -> &str {
        "api_request"
    }

    fn description(&self) -> &str {
        "Makes an API request"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from api_request (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct APISchemaTool;

#[async_trait]
impl BaseTool for APISchemaTool {
    fn name(&self) -> &str {
        "api_schema"
    }

    fn description(&self) -> &str {
        "Returns the schema of an API"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from api_schema (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct OpenAPIToolkit;

impl OpenAPIToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for OpenAPIToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![Arc::new(APIRequestTool) as Arc<dyn BaseTool>, Arc::new(APISchemaTool)]
    }

    fn name(&self) -> &str {
        "openapi"
    }
}
