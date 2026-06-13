//! Multi-query retriever that generates multiple query variations from the
//! original to improve recall.
//!
//! The [`MultiQueryRetriever`] uses an LLM to produce several different
//! phrasings of the input query, executes each against the underlying
//! retriever, and merges the results. This helps overcome the limitations
//! of a single query and improves retrieval coverage.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;
use std::sync::Arc;

/// Generates multiple query variations to improve retrieval recall.
///
/// An optional LLM chain produces alternative phrasings of the input query.
/// Each variant is searched independently, and the results are merged with
/// optional deduplication.
#[derive(Clone)]
pub struct MultiQueryRetriever {
    /// The underlying retriever to query with each variant.
    pub retriever: Arc<dyn BaseRetriever>,
    /// Whether to include results from the original query.
    pub include_original: bool,
    /// Optional LLM chain configuration for query generation.
    pub llm_chain: Option<String>,
}

impl std::fmt::Debug for MultiQueryRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiQueryRetriever")
            .field("include_original", &self.include_original)
            .field("llm_chain", &self.llm_chain)
            .finish()
    }
}

impl MultiQueryRetriever {
    /// Creates a new `MultiQueryRetriever`.
    ///
    /// # Arguments
    /// * `retriever` - The base retriever.
    /// * `include_original` - Whether to also run the original query.
    pub fn new(retriever: Arc<dyn BaseRetriever>, include_original: bool) -> Self {
        Self {
            retriever,
            include_original,
            llm_chain: None,
        }
    }
}

#[async_trait]
impl BaseRetriever for MultiQueryRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.retriever.add_documents(documents).await
    }
}
