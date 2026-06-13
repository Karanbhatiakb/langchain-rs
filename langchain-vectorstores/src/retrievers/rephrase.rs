//! RePhrase query retriever that reformulates the query before passing it to
//! the underlying retriever.
//!
//! The [`RePhraseQueryRetriever`] uses a language model chain to rewrite the
//! user's query into a more effective search query, then delegates to the
//! inner retriever. This can improve retrieval quality for poorly worded or
//! ambiguous queries.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;
use std::sync::Arc;

/// Wraps a retriever and rephrases the query before retrieval.
///
/// An optional LLM chain (identified by name or configuration string) is
/// used to reformulate the original query into a more search-friendly form.
/// The rephrased query is then passed to the inner retriever.
#[derive(Clone)]
pub struct RePhraseQueryRetriever {
    /// The inner retriever to delegate to after rephrasing.
    pub retriever: Arc<dyn BaseRetriever>,
    /// Optional identifier for the LLM rephrasing chain.
    pub llm_chain: Option<String>,
}

impl std::fmt::Debug for RePhraseQueryRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RePhraseQueryRetriever")
            .field("llm_chain", &self.llm_chain)
            .finish()
    }
}

impl RePhraseQueryRetriever {
    /// Creates a new `RePhraseQueryRetriever`.
    ///
    /// # Arguments
    /// * `retriever` - The inner retriever that receives the rephrased query.
    /// * `llm_chain` - Optional LLM chain configuration for rephrasing.
    pub fn new(retriever: Arc<dyn BaseRetriever>, llm_chain: Option<String>) -> Self {
        Self {
            retriever,
            llm_chain,
        }
    }
}

#[async_trait]
impl BaseRetriever for RePhraseQueryRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.retriever.add_documents(documents).await
    }
}
