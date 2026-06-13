//! VectorStoreQA tool for asking questions against a vector store.
//!
//! This tool takes a natural-language question, retrieves relevant
//! documents from a configured vector store, and returns an answer
//! synthesised from those documents.

use async_trait::async_trait;
use super::traits::{BaseTool, ToolResult};

/// Tool that answers questions by querying a vector store.
///
/// # Stub
///
/// This is a stub implementation. Configure a real vector-store retriever
/// and LLM chain to enable actual Q&A.
#[derive(Debug, Clone)]
pub struct VectorStoreQA;

impl VectorStoreQA {
    /// Creates a new [`VectorStoreQA`] tool.
    pub fn new() -> Self {
        Self
    }
}

impl Default for VectorStoreQA {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for VectorStoreQA {
    fn name(&self) -> &str {
        "vectorstore_qa"
    }

    fn description(&self) -> &str {
        "Answers questions against a vector store by retrieving relevant documents and synthesising an answer"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from vectorstore_qa (stub)".to_string())
    }
}
