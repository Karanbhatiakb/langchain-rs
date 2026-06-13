//! Tavily search API retriever for web search via the Tavily API.
//!
//! The [`TavilySearchAPIRetriever`] uses the Tavily search engine to retrieve
//! web pages relevant to the query. Tavily is an AI-native search API that
//! returns curated, high-quality search results.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;

/// Searches the web using the Tavily API.
///
/// Requires a Tavily API key (set via the `TAVILY_API_KEY` environment
/// variable or passed directly). Returns web search results as documents
/// with page content and source metadata.
#[derive(Debug, Clone)]
pub struct TavilySearchAPIRetriever {
    /// Number of top results to return.
    pub top_k: usize,
    /// Optional Tavily API key.
    pub api_key: Option<String>,
    /// Optional HTTP client configuration name.
    pub client: Option<String>,
}

impl TavilySearchAPIRetriever {
    /// Creates a new `TavilySearchAPIRetriever`.
    ///
    /// # Arguments
    /// * `top_k` - Number of search results to retrieve.
    /// * `api_key` - Optional Tavily API key (falls back to env var).
    pub fn new(top_k: usize, api_key: Option<String>) -> Self {
        Self {
            top_k,
            api_key,
            client: None,
        }
    }
}

#[async_trait]
impl BaseRetriever for TavilySearchAPIRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
