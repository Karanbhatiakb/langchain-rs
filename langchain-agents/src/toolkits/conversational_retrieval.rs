use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct RetrieverTool;

#[async_trait]
impl BaseTool for RetrieverTool {
    fn name(&self) -> &str {
        "retriever"
    }

    fn description(&self) -> &str {
        "Searches for relevant documents"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("No documents found (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct ConversationalRetrievalToolkit {
    #[allow(dead_code)]
    retriever_name: String,
}

impl ConversationalRetrievalToolkit {
    pub fn new() -> Self {
        Self {
            retriever_name: "default".to_string(),
        }
    }
}

impl BaseToolkit for ConversationalRetrievalToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![Arc::new(RetrieverTool)]
    }

    fn name(&self) -> &str {
        "conversational_retrieval"
    }
}
