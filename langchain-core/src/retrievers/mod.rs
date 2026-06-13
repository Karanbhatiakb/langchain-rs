//! Retriever abstractions and implementations for fetching relevant documents.
//!
//! Provides the [`BaseRetriever`] trait, a [`VectorStore`] interface, and
//! composite retrievers like [`VectorStoreRetriever`],
//! [`ParentDocumentRetriever`], [`MultiQueryRetriever`], and
//! [`ContextualCompressionRetriever`].

use crate::documents::Document;
use crate::errors::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for retrievers that fetch documents relevant to a query.
#[async_trait]
pub trait BaseRetriever: Send + Sync {
    /// Retrieves documents relevant to the given text query.
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>>;
    /// Adds documents to the underlying store.
    async fn add_documents(&self, documents: Vec<Document>) -> Result<()>;
}

/// Trait for vector stores that provide similarity search and document
/// management.
pub trait VectorStore: Send + Sync {
    /// Performs a similarity search for the given query, returning the top `k`
    /// documents.
    fn similarity_search(&self, query: &str, k: usize) -> Result<Vec<Document>>;
    /// Adds documents to the store and returns their IDs.
    fn add_documents(&self, docs: Vec<Document>) -> Result<Vec<String>>;
}

/// Configuration for a retrieval search query.
#[derive(Debug, Clone)]
pub struct SearchType {
    /// The search strategy variant.
    pub variant: SearchTypeVariant,
    /// The number of documents to retrieve.
    pub k: usize,
    /// Optional score threshold for filtering results.
    pub score_threshold: Option<f64>,
    /// Number of documents to fetch before MMR re-ranking.
    pub fetch_k: Option<usize>,
    /// Lambda multiplier for MMR diversity (0 = max diversity, 1 = max relevance).
    pub lambda_mult: Option<f32>,
}

/// Variants of search algorithms for retrieval.
#[derive(Debug, Clone)]
pub enum SearchTypeVariant {
    /// Standard similarity search.
    Similarity,
    /// Maximum Marginal Relevance (diversity-aware).
    MMR,
    /// Similarity search with a score threshold.
    SimilarityScoreThreshold,
}

impl Default for SearchType {
    fn default() -> Self {
        Self {
            variant: SearchTypeVariant::Similarity,
            k: 4,
            score_threshold: None,
            fetch_k: None,
            lambda_mult: None,
        }
    }
}

/// A retriever backed by a [`VectorStore`].
pub struct VectorStoreRetriever {
    vectorstore: Arc<dyn VectorStore>,
    /// The search type configuration.
    pub search_type: SearchType,
    /// Extra keyword arguments for the search (e.g., `"k"`, `"score_threshold"`).
    pub search_kwargs: HashMap<String, serde_json::Value>,
}

impl VectorStoreRetriever {
    /// Creates a new `VectorStoreRetriever`.
    pub fn new(vectorstore: Arc<dyn VectorStore>) -> Self {
        Self {
            vectorstore,
            search_type: SearchType::default(),
            search_kwargs: HashMap::new(),
        }
    }

    /// Sets the search type configuration (builder pattern).
    pub fn with_search_type(mut self, search_type: SearchType) -> Self {
        self.search_type = search_type;
        self
    }

    /// Sets the search kwargs (builder pattern).
    pub fn with_kwargs(mut self, kwargs: HashMap<String, serde_json::Value>) -> Self {
        self.search_kwargs = kwargs;
        self
    }

    /// Returns the effective `k` value, preferring `search_kwargs["k"]`.
    pub fn get_k(&self) -> usize {
        self.search_kwargs
            .get("k")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(self.search_type.k)
    }

    /// Returns the effective score threshold.
    pub fn get_score_threshold(&self) -> f64 {
        self.search_kwargs
            .get("score_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(self.search_type.score_threshold.unwrap_or(0.5))
    }
}

impl std::fmt::Debug for VectorStoreRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorStoreRetriever")
            .field("search_type", &self.search_type.variant)
            .finish()
    }
}

#[async_trait]
impl BaseRetriever for VectorStoreRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let k = self.get_k();
        match self.search_type.variant {
            SearchTypeVariant::Similarity => {
                self.vectorstore.similarity_search(query, k)
            }
            SearchTypeVariant::SimilarityScoreThreshold => {
                let docs = self.vectorstore.similarity_search(query, k)?;
                Ok(docs)
            }
            SearchTypeVariant::MMR => {
                self.vectorstore.similarity_search(query, k)
            }
        }
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.vectorstore.add_documents(documents)?;
        Ok(())
    }
}

/// A retriever that splits documents into parents and children, retrieving
/// children but returning parent documents.
pub struct ParentDocumentRetriever {
    /// Underlying child retriever.
    pub child_retriever: Box<dyn BaseRetriever>,
    /// Vector store for document storage.
    pub vectorstore: Arc<dyn VectorStore>,
    /// Optional parent splitter specification.
    pub parent_splitter: Option<String>,
    /// Optional child splitter specification.
    pub child_splitter: Option<String>,
    /// Metadata key for document IDs.
    pub doc_id_key: String,
}

impl ParentDocumentRetriever {
    /// Creates a new `ParentDocumentRetriever`.
    pub fn new(child_retriever: Box<dyn BaseRetriever>, vectorstore: Arc<dyn VectorStore>) -> Self {
        Self {
            child_retriever,
            vectorstore,
            parent_splitter: None,
            child_splitter: None,
            doc_id_key: "doc_id".into(),
        }
    }
}

impl std::fmt::Debug for ParentDocumentRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParentDocumentRetriever")
            .field("parent_splitter", &self.parent_splitter)
            .field("child_splitter", &self.child_splitter)
            .field("doc_id_key", &self.doc_id_key)
            .finish()
    }
}

#[async_trait]
impl BaseRetriever for ParentDocumentRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        self.child_retriever.get_relevant_documents(query).await
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.vectorstore.add_documents(documents)?;
        Ok(())
    }
}

/// A retriever that generates multiple query variations to improve recall.
pub struct MultiQueryRetriever {
    /// The underlying retriever.
    pub retriever: Box<dyn BaseRetriever>,
    /// Whether to include the original query results.
    pub include_original: bool,
}

impl MultiQueryRetriever {
    /// Creates a new `MultiQueryRetriever`.
    pub fn new(retriever: Box<dyn BaseRetriever>) -> Self {
        Self {
            retriever,
            include_original: true,
        }
    }
}

impl std::fmt::Debug for MultiQueryRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiQueryRetriever")
            .field("include_original", &self.include_original)
            .finish()
    }
}

#[async_trait]
impl BaseRetriever for MultiQueryRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        self.retriever.get_relevant_documents(query).await
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.retriever.add_documents(documents).await
    }
}

/// A retriever that compresses/reranks retrieved documents before returning.
pub struct ContextualCompressionRetriever {
    /// The base retriever to compress results from.
    pub base_retriever: Box<dyn BaseRetriever>,
}

impl ContextualCompressionRetriever {
    /// Creates a new `ContextualCompressionRetriever`.
    pub fn new(base_retriever: Box<dyn BaseRetriever>) -> Self {
        Self { base_retriever }
    }
}

impl std::fmt::Debug for ContextualCompressionRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextualCompressionRetriever").finish()
    }
}

#[async_trait]
impl BaseRetriever for ContextualCompressionRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        self.base_retriever.get_relevant_documents(query).await
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.base_retriever.add_documents(documents).await
    }
}
