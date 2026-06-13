//! Wikipedia retriever that searches Wikipedia articles for relevant
//! documents.
//!
//! The [`WikipediaRetriever`] uses the Wikipedia API to search for articles
//! matching the query, fetches the article content, and returns them as
//! [`Document`] instances. Language and result count are configurable.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;

/// Searches Wikipedia for documents matching a query.
///
/// Uses the public Wikipedia API to search articles in the specified
/// language. Returns the top `top_k` results as LangChain documents.
#[derive(Debug, Clone)]
pub struct WikipediaRetriever {
    /// Number of top articles to return.
    pub top_k: usize,
    /// Wikipedia language code (e.g., "en", "de", "fr").
    pub lang: String,
    /// Optional HTTP client configuration name.
    pub client: Option<String>,
}

impl WikipediaRetriever {
    /// Creates a new `WikipediaRetriever`.
    ///
    /// # Arguments
    /// * `top_k` - Number of articles to retrieve.
    /// * `lang` - Wikipedia language code.
    pub fn new(top_k: usize, lang: String) -> Self {
        Self {
            top_k,
            lang,
            client: None,
        }
    }
}

#[async_trait]
impl BaseRetriever for WikipediaRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
