//! Vector store traits, search types, and an in-memory implementation.
//!
//! Defines the [`VectorStore`] trait — the core abstraction for storing
//! documents alongside their embeddings and performing similarity-based
//! retrieval — plus [`InMemoryVectorStore`] for testing and simple
//! workloads, and the [`SearchType`] enum for parameterised search.
//!
//! # Example
//! ```ignore
//! use langchain_core::vectorstores::{InMemoryVectorStore, VectorStore};
//! use langchain_core::embeddings::FakeEmbeddings;
//! use langchain_core::documents::Document;
//!
//! async fn example() {
//!     let embeddings = FakeEmbeddings::new(64);
//!     let store = InMemoryVectorStore::new(Box::new(embeddings));
//!     let docs = vec![Document::new("hello world")];
//!     let ids = store.add_documents(&docs).await.unwrap();
//!     let results = store.similarity_search("hello", 1).await.unwrap();
//! }
//! ```

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::documents::Document;
use crate::embeddings::Embeddings;
use crate::errors::{ChainError, Result};

// ---------------------------------------------------------------------------
// Search type parameter structs
// ---------------------------------------------------------------------------

/// Parameters for plain similarity search.
#[derive(Debug, Clone)]
pub struct SimilaritySearchParams {
    /// Number of documents to return.
    pub k: usize,
    /// Optional relevance-score threshold (0.0 – 1.0).  Documents with a
    /// score below this value are excluded from the results.
    pub score_threshold: Option<f64>,
}

impl SimilaritySearchParams {
    /// Create a new parameter set with the given `k` and no score threshold.
    pub fn new(k: usize) -> Self {
        Self {
            k,
            score_threshold: None,
        }
    }
}

/// Parameters for Maximal Marginal Relevance (MMR) search.
#[derive(Debug, Clone)]
pub struct MMRSearchParams {
    /// Number of documents to return.
    pub k: usize,
    /// Number of documents to fetch before running the MMR algorithm.
    pub fetch_k: usize,
    /// Diversity trade-off in `[0, 1]`.  `0` = maximum diversity,
    /// `1` = maximum similarity (minimum diversity).
    pub lambda_mult: f64,
}

impl MMRSearchParams {
    /// Create a new parameter set with the given `k`, `fetch_k`, and
    /// `lambda_mult`.
    pub fn new(k: usize, fetch_k: usize, lambda_mult: f64) -> Self {
        Self {
            k,
            fetch_k,
            lambda_mult,
        }
    }
}

/// Enum describing the type of search to perform.
///
/// Used by higher-level APIs (e.g. retrievers) to dispatch to the right
/// search method on a [`VectorStore`].
#[derive(Debug, Clone)]
pub enum SearchType {
    /// Plain cosine-similarity search.
    Similarity(SimilaritySearchParams),
    /// Maximal Marginal Relevance search (balances relevance and diversity).
    MMR(MMRSearchParams),
}

// ---------------------------------------------------------------------------
// VectorStore trait
// ---------------------------------------------------------------------------

/// Trait for vector stores that index documents by their embeddings and
/// support similarity-based retrieval.
///
/// A vector store is responsible for:
/// 1.  Persisting documents alongside their embedding vectors.
/// 2.  Answering similarity queries (by text or by embedding vector).
/// 3.  Optionally supporting Maximal Marginal Relevance (MMR) search for
///     diversity-aware retrieval.
///
/// All implementors must be [`Send`]+[`Sync`]+`'static` and use
/// `#[async_trait]` on the `impl` block.
///
/// Default implementations are provided for methods that can be expressed
/// in terms of the primitive operations, so implementors only need to
/// implement `add_documents`, `add_embeddings`, `delete`, and
/// `similarity_search_by_vector`.
#[async_trait]
pub trait VectorStore: Send + Sync + 'static {
    /// Add documents to the store.
    ///
    /// The store will call its underlying [`Embeddings`] to produce vectors
    /// for each document, then persist them.
    ///
    /// # Arguments
    /// * `docs` — documents to add.
    ///
    /// # Returns
    /// A list of string IDs assigned to the added documents.
    async fn add_documents(&self, docs: &[Document]) -> Result<Vec<String>>;

    /// Add pre-computed (document, embedding) pairs to the store.
    ///
    /// Use this when the caller already has embeddings available (e.g. from
    /// a cache) to avoid recomputation.
    ///
    /// # Arguments
    /// * `embeddings` — slice of `(Document, Vec<f64>)` pairs.
    ///
    /// # Returns
    /// A list of string IDs assigned to the added documents.
    async fn add_embeddings(&self, embeddings: &[(Document, Vec<f64>)]) -> Result<Vec<String>>;

    /// Delete documents by their IDs.
    ///
    /// # Arguments
    /// * `ids` — the document IDs to remove.
    ///
    /// # Errors
    /// Returns [`ChainError::VectorStoreError`] if the underlying store
    /// reports a failure.
    async fn delete(&self, ids: &[String]) -> Result<()>;

    /// Return the `k` documents most similar to the query text.
    ///
    /// The default implementation embeds the query and delegates to
    /// [`similarity_search_by_vector`](`VectorStore::similarity_search_by_vector`).
    async fn similarity_search(&self, query: &str, k: usize) -> Result<Vec<Document>> {
        let embedding = self.embed_query(query).await?;
        self.similarity_search_by_vector(&embedding, k).await
    }

    /// Return the `k` documents most similar to the given embedding vector.
    ///
    /// This is the core primitive that concrete stores must implement.
    async fn similarity_search_by_vector(
        &self,
        embedding: &[f64],
        k: usize,
    ) -> Result<Vec<Document>>;

    /// Return the `k` most-similar documents together with their similarity
    /// scores.
    ///
    /// The default implementation embeds the query, fetches `k` results with
    /// scores via [`similarity_search_by_vector_with_scores`], and strips
    /// scores below the optional threshold.
    async fn similarity_search_with_score(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<(Document, f64)>> {
        let embedding = self.embed_query(query).await?;
        self.similarity_search_by_vector_with_score(&embedding, k)
            .await
    }

    /// Return the `k` most-similar documents to an embedding vector,
    /// together with their similarity scores.
    ///
    /// The default implementation calls
    /// [`similarity_search_by_vector`](`VectorStore::similarity_search_by_vector`)
    /// and assigns a score of `0.0` to each result.  Stores that can
    /// efficiently produce scores should override this method.
    async fn similarity_search_by_vector_with_score(
        &self,
        embedding: &[f64],
        k: usize,
    ) -> Result<Vec<(Document, f64)>> {
        let docs = self.similarity_search_by_vector(embedding, k).await?;
        Ok(docs.into_iter().map(|d| (d, 0.0)).collect())
    }

    /// Return documents selected using Maximal Marginal Relevance.
    ///
    /// MMR optimizes for similarity to the query **and** diversity among
    /// the selected documents, controlled by `lambda_mult`.
    ///
    /// The default implementation embeds the query and delegates to
    /// [`max_marginal_relevance_search_by_vector`].
    async fn max_marginal_relevance_search(
        &self,
        query: &str,
        k: usize,
        fetch_k: usize,
        lambda_mult: f64,
    ) -> Result<Vec<Document>> {
        let embedding = self.embed_query(query).await?;
        self.max_marginal_relevance_search_by_vector(&embedding, k, fetch_k, lambda_mult)
            .await
    }

    /// Return documents selected using Maximal Marginal Relevance from an
    /// embedding vector.
    ///
    /// The default implementation falls back to plain similarity search,
    /// because a correct MMR implementation requires access to all stored
    /// vectors.  Stores that can iterate over stored embeddings should
    /// override this.
    async fn max_marginal_relevance_search_by_vector(
        &self,
        embedding: &[f64],
        k: usize,
        _fetch_k: usize,
        _lambda_mult: f64,
    ) -> Result<Vec<Document>> {
        self.similarity_search_by_vector(embedding, k).await
    }
}

/// Helper: embed a query using the store's embedded [`Embeddings`] instance.
/// This is factored out so trait-default methods can share the logic without
/// requiring `self` to be a concrete type.
async fn embed_query(embedder: &(dyn Embeddings + Send + Sync), query: &str) -> Result<Vec<f64>> {
    embedder.embed_query(query).await
}

// ---------------------------------------------------------------------------
// InMemoryVectorStore
// ---------------------------------------------------------------------------

/// A single entry stored in [`InMemoryVectorStore`].
#[derive(Debug, Clone)]
struct StoreEntry {
    document: Document,
    embedding: Vec<f64>,
}

/// A simple in-memory vector store backed by a `HashMap`.
///
/// Similarity is computed using cosine similarity.  This store is suitable
/// for testing, prototyping, and small workloads.  It is **not** persisted
/// to disk.
///
/// # Example
/// ```ignore
/// use langchain_core::vectorstores::InMemoryVectorStore;
/// use langchain_core::embeddings::FakeEmbeddings;
/// use langchain_core::documents::Document;
/// use langchain_core::vectorstores::VectorStore;
///
/// let emb = FakeEmbeddings::new(64);
/// let store = InMemoryVectorStore::new(Box::new(emb));
/// ```
pub struct InMemoryVectorStore {
    store: parking_lot::RwLock<HashMap<String, StoreEntry>>,
    embeddings: Arc<dyn Embeddings>,
}

impl InMemoryVectorStore {
    /// Create a new, empty `InMemoryVectorStore` backed by the given
    /// embedding model.
    pub fn new(embeddings: Box<dyn Embeddings>) -> Self {
        Self {
            store: parking_lot::RwLock::new(HashMap::new()),
            embeddings: Arc::from(embeddings),
        }
    }
}

/// Compute the cosine similarity between two vectors.
///
/// Returns `0.0` if either vector has zero norm.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

/// Select indices using the Maximal Marginal Relevance algorithm.
///
/// Returns at most `k` indices from `candidates` (in order), balancing
/// relevance (`sim_to_query[i]`) and diversity (pairwise similarity of
/// selected candidates), controlled by `lambda_mult`.
fn maximal_marginal_relevance(
    sim_to_query: &[f64],
    embeddings: &[Vec<f64>],
    k: usize,
    lambda_mult: f64,
) -> Vec<usize> {
    let n = sim_to_query.len();
    if n == 0 || k == 0 {
        return Vec::new();
    }

    let mut selected_indices: Vec<usize> = Vec::new();
    let mut remaining: Vec<usize> = (0..n).collect();

    for _ in 0..k {
        let best_idx = remaining.iter().copied().max_by(|&i, &j| {
            let score_i = lambda_mult * sim_to_query[i]
                - (1.0 - lambda_mult)
                    * selected_indices
                        .iter()
                        .map(|&s| cosine_similarity(&embeddings[i], &embeddings[s]))
                        .fold(0.0_f64, f64::max);
            let score_j = lambda_mult * sim_to_query[j]
                - (1.0 - lambda_mult)
                    * selected_indices
                        .iter()
                        .map(|&s| cosine_similarity(&embeddings[j], &embeddings[s]))
                        .fold(0.0_f64, f64::max);
            score_i
                .partial_cmp(&score_j)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(idx) = best_idx {
            selected_indices.push(idx);
            remaining.retain(|&x| x != idx);
        } else {
            break;
        }
    }

    selected_indices
}

// We need a small helper trait method for embedding queries inside the
// VectorStore defaults.  Since the trait object is behind Arc we just call
// it directly from the InMemoryVectorStore impls.
impl InMemoryVectorStore {
    /// Embed a query using the store's embedding model.
    async fn do_embed_query(&self, query: &str) -> Result<Vec<f64>> {
        self.embeddings.embed_query(query).await
    }

    /// Embed a batch of documents using the store's embedding model.
    async fn do_embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f64>>> {
        self.embeddings.embed_documents(texts).await
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_documents(&self, docs: &[Document]) -> Result<Vec<String>> {
        let texts: Vec<String> = docs.iter().map(|d| d.page_content.clone()).collect();
        let vectors = self.do_embed_documents(&texts).await?;
        let mut ids = Vec::with_capacity(docs.len());
        let mut store = self.store.write();
        for (doc, embedding) in docs.iter().zip(vectors.iter()) {
            let id = uuid::Uuid::new_v4().to_string();
            store.insert(
                id.clone(),
                StoreEntry {
                    document: doc.clone(),
                    embedding: embedding.clone(),
                },
            );
            ids.push(id);
        }
        Ok(ids)
    }

    async fn add_embeddings(&self, embeddings: &[(Document, Vec<f64>)]) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(embeddings.len());
        let mut store = self.store.write();
        for (doc, embedding) in embeddings {
            let id = uuid::Uuid::new_v4().to_string();
            store.insert(
                id.clone(),
                StoreEntry {
                    document: doc.clone(),
                    embedding: embedding.clone(),
                },
            );
            ids.push(id);
        }
        Ok(ids)
    }

    async fn delete(&self, ids: &[String]) -> Result<()> {
        let mut store = self.store.write();
        for id in ids {
            if store.remove(id).is_none() {
                return Err(ChainError::VectorStoreError(format!(
                    "Document with id '{}' not found",
                    id
                )));
            }
        }
        Ok(())
    }

    async fn similarity_search(&self, query: &str, k: usize) -> Result<Vec<Document>> {
        let embedding = self.do_embed_query(query).await?;
        self.similarity_search_by_vector(&embedding, k).await
    }

    async fn similarity_search_by_vector(
        &self,
        embedding: &[f64],
        k: usize,
    ) -> Result<Vec<Document>> {
        let results = self.search_by_vector_with_score(embedding, k);
        Ok(results.into_iter().map(|(doc, _)| doc).collect())
    }

    async fn similarity_search_with_score(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<(Document, f64)>> {
        let embedding = self.do_embed_query(query).await?;
        self.similarity_search_by_vector_with_score(&embedding, k).await
    }

    async fn similarity_search_by_vector_with_score(
        &self,
        embedding: &[f64],
        k: usize,
    ) -> Result<Vec<(Document, f64)>> {
        Ok(self.search_by_vector_with_score(embedding, k))
    }

    async fn max_marginal_relevance_search(
        &self,
        query: &str,
        k: usize,
        fetch_k: usize,
        lambda_mult: f64,
    ) -> Result<Vec<Document>> {
        let embedding = self.do_embed_query(query).await?;
        self.max_marginal_relevance_search_by_vector(&embedding, k, fetch_k, lambda_mult)
            .await
    }

    async fn max_marginal_relevance_search_by_vector(
        &self,
        embedding: &[f64],
        k: usize,
        fetch_k: usize,
        lambda_mult: f64,
    ) -> Result<Vec<Document>> {
        let store = self.store.read();
        if store.is_empty() {
            return Ok(Vec::new());
        }

        let entries: Vec<&StoreEntry> = store.values().collect();
        let sims: Vec<f64> = entries
            .iter()
            .map(|e| cosine_similarity(embedding, &e.embedding))
            .collect();
        let emb_vecs: Vec<Vec<f64>> = entries.iter().map(|e| e.embedding.clone()).collect();
        drop(store);

        let chosen = maximal_marginal_relevance(&sims, &emb_vecs, k, lambda_mult);

        let store = self.store.read();
        let all_entries: Vec<&StoreEntry> = store.values().collect();
        let mut result = Vec::with_capacity(chosen.len());
        for idx in chosen {
            if idx < all_entries.len() {
                result.push(all_entries[idx].document.clone());
            }
        }
        Ok(result)
    }
}

impl InMemoryVectorStore {
    /// Internal helper: search by vector with scores (no async needed).
    fn search_by_vector_with_score(
        &self,
        embedding: &[f64],
        k: usize,
    ) -> Vec<(Document, f64)> {
        let store = self.store.read();
        if store.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(Document, f64)> = store
            .values()
            .map(|entry| {
                let score = cosine_similarity(embedding, &entry.embedding);
                (entry.document.clone(), score)
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        scored
    }
}

// ---------------------------------------------------------------------------
// Make VectorStore defaults work by delegating to InMemoryVectorStore helpers
// ---------------------------------------------------------------------------

/// Provide a bridge from the trait-default `similarity_search` to the concrete
/// implementation.  We implement the default-method helper outside the trait
/// by embedding the query in a free function used by the trait's default body.
///
/// The trait's default `similarity_search` calls `self.embed_query`, which
/// doesn't exist on `dyn VectorStore`.  We work around this by overriding
/// every method that needs embedding so the defaults are never used for
/// `InMemoryVectorStore`.  (All methods are already overridden above.)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::FakeEmbeddings;

    #[tokio::test]
    async fn test_add_and_search() {
        let embeddings = FakeEmbeddings::new(32);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let docs = vec![
            Document::new("hello world"),
            Document::new("foo bar"),
        ];
        let ids = store.add_documents(&docs).await.unwrap();
        assert_eq!(ids.len(), 2);

        let results = store.similarity_search("hello", 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].page_content, "hello world");
    }

    #[tokio::test]
    async fn test_add_embeddings() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let doc = Document::new("test");
        let emb = vec![0.1; 16];
        let ids = store
            .add_embeddings(&[(doc, emb)])
            .await
            .unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_delete() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let doc = Document::new("to be deleted");
        let ids = store.add_documents(&[doc]).await.unwrap();
        store.delete(&ids).await.unwrap();
        let results = store.similarity_search("deleted", 1).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_similarity_search_with_score() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        store
            .add_documents(&[Document::new("hello")])
            .await
            .unwrap();
        let results = store
            .similarity_search_with_score("hello", 1)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        let (_, score) = results[0];
        assert!(score > 0.0);
    }

    #[tokio::test]
    async fn test_mmr_search() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let docs = vec![
            Document::new("alpha"),
            Document::new("beta"),
            Document::new("gamma"),
        ];
        store.add_documents(&docs).await.unwrap();
        let results = store
            .max_marginal_relevance_search("alpha", 2, 3, 0.5)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-9);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 1e-9);
    }

    #[tokio::test]
    async fn test_empty_store_search() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let results = store
            .similarity_search("anything", 5)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_type_params() {
        let sim = SimilaritySearchParams::new(10);
        assert_eq!(sim.k, 10);
        assert!(sim.score_threshold.is_none());

        let mmr = MMRSearchParams::new(5, 20, 0.7);
        assert_eq!(mmr.k, 5);
        assert_eq!(mmr.fetch_k, 20);
        assert!((mmr.lambda_mult - 0.7).abs() < 1e-9);
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let result = store.delete(&["nonexistent".into()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_similarity_search_by_vector() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        store
            .add_documents(&[Document::new("hello")])
            .await
            .unwrap();
        let query_emb = vec![0.1; 16];
        let results = store
            .similarity_search_by_vector(&query_emb, 1)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_max_marginal_relevance_empty_store() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let results = store
            .max_marginal_relevance_search("anything", 3, 5, 0.5)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_max_marginal_relevance_single_doc() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        store
            .add_documents(&[Document::new("only")])
            .await
            .unwrap();
        let results = store
            .max_marginal_relevance_search("only", 3, 5, 0.5)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_cosine_similarity_zero_norm() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((cosine_similarity(&a, &b)).abs() < f64::EPSILON);
        assert!((cosine_similarity(&b, &a)).abs() < f64::EPSILON);
        assert!((cosine_similarity(&a, &a)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_maximal_marginal_relevance_empty() {
        let result = maximal_marginal_relevance(&[], &[], 5, 0.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_maximal_marginal_relevance_zero_k() {
        let sims = vec![0.9, 0.8, 0.7];
        let embs = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![0.5, 0.5]];
        let result = maximal_marginal_relevance(&sims, &embs, 0, 0.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_search_type_enum() {
        let sim = SearchType::Similarity(SimilaritySearchParams::new(5));
        let mmr = SearchType::MMR(MMRSearchParams::new(3, 10, 0.8));
        match sim {
            SearchType::Similarity(ref p) => assert_eq!(p.k, 5),
            _ => panic!("expected Similarity"),
        }
        match mmr {
            SearchType::MMR(ref p) => {
                assert_eq!(p.k, 3);
                assert!((p.lambda_mult - 0.8).abs() < 1e-9);
            }
            _ => panic!("expected MMR"),
        }
    }

    #[test]
    fn test_vector_store_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<InMemoryVectorStore>();
        assert_sync::<InMemoryVectorStore>();
    }

    #[test]
    fn test_similarity_search_params_defaults() {
        let params = SimilaritySearchParams::new(5);
        assert_eq!(params.k, 5);
        assert!(params.score_threshold.is_none());
    }

    #[test]
    fn test_mmr_search_params_defaults() {
        let params = MMRSearchParams::new(3, 10, 0.5);
        assert_eq!(params.k, 3);
        assert_eq!(params.fetch_k, 10);
        assert!((params.lambda_mult - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_search_type_debug() {
        let sim = SearchType::Similarity(SimilaritySearchParams::new(3));
        let debug = format!("{:?}", sim);
        assert!(debug.contains("Similarity"));
    }

    #[tokio::test]
    async fn test_add_documents_then_search_by_vector() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let ids = store
            .add_documents(&[Document::new("test")])
            .await
            .unwrap();
        assert_eq!(ids.len(), 1);

        let query = vec![0.5; 16];
        let results = store.similarity_search_by_vector(&query, 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].page_content, "test");
    }

    #[tokio::test]
    async fn test_mmr_search_with_score() {
        let embeddings = FakeEmbeddings::new(16);
        let store = InMemoryVectorStore::new(Box::new(embeddings));
        let docs = vec![
            Document::new("a"),
            Document::new("b"),
            Document::new("c"),
        ];
        store.add_documents(&docs).await.unwrap();
        let results = store
            .max_marginal_relevance_search("a", 3, 5, 0.5)
            .await
            .unwrap();
        assert_eq!(results.len(), 3);
    }
}
