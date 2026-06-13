use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct VectorStoreSearchTool;

#[async_trait]
impl BaseTool for VectorStoreSearchTool {
    fn name(&self) -> &str {
        "vectorstore_search"
    }

    fn description(&self) -> &str {
        "Searches a vector store"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from vectorstore_search (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct VectorStoreAddTool;

#[async_trait]
impl BaseTool for VectorStoreAddTool {
    fn name(&self) -> &str {
        "vectorstore_add"
    }

    fn description(&self) -> &str {
        "Adds documents to a vector store"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from vectorstore_add (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct VectorStoreToolkit;

impl VectorStoreToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for VectorStoreToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![Arc::new(VectorStoreSearchTool) as Arc<dyn BaseTool>, Arc::new(VectorStoreAddTool)]
    }

    fn name(&self) -> &str {
        "vectorstore"
    }
}
