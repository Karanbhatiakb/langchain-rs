//! Embedding model traits and test utilities.
//!
//! Defines the [`Embeddings`] trait — the core abstraction for converting text
//! into dense vector representations — plus [`FakeEmbeddings`] and
//! [`DeterministicFakeEmbedding`] for testing and prototyping without a real
//! embedding provider.
//!
//! # Example
//! ```ignore
//! use langchain_core::embeddings::Embeddings;
//!
//! async fn example<E: Embeddings>(embedder: &E) {
//!     let docs = embedder.embed_documents(&["hello".into(), "world".into()]).await;
//!     let query = embedder.embed_query("hello").await;
//! }
//! ```

use async_trait::async_trait;
use crate::errors::{ChainError, Result};

/// Trait for embedding models that convert text into dense vector representations.
///
/// Text embedding models map text to a point in n-dimensional space so that
/// similar texts are close together. The exact notion of "similarity" and
/// "distance" depends on the specific model.
///
/// The trait distinguishes between embedding *documents* (batch) and embedding
/// a single *query*, because some providers use different endpoints or
/// normalization for the two use-cases.
///
/// An optional [`embed_image`](`Embeddings::embed_image`) method is provided
/// for multimodal models; the default implementation returns
/// [`ChainError::EmbeddingError`].
///
/// # Implementing
///
/// All implementors must be [`Send`]+[`Sync`]+`'static` and use
/// `#[async_trait]` on the `impl` block.
#[async_trait]
pub trait Embeddings: Send + Sync + 'static {
    /// Embed a batch of documents, returning one vector per document.
    ///
    /// # Arguments
    /// * `texts` — the document strings to embed.
    ///
    /// # Returns
    /// A `Vec` of embedding vectors in the same order as `texts`.
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f64>>>;

    /// Embed a single query string.
    ///
    /// This may use a different endpoint or normalization strategy than
    /// [`embed_documents`](`Embeddings::embed_documents`).
    ///
    /// # Arguments
    /// * `text` — the query string to embed.
    ///
    /// # Returns
    /// A single embedding vector.
    async fn embed_query(&self, text: &str) -> Result<Vec<f64>>;

    /// Embed raw image bytes into a vector.
    ///
    /// This is an optional method — only multimodal embedding models support
    /// it. The default implementation returns an
    /// [`ChainError::EmbeddingError`].
    ///
    /// # Arguments
    /// * `image_data` — raw bytes of the image (e.g. PNG, JPEG).
    ///
    /// # Returns
    /// A single embedding vector on success, or an error if the model does
    /// not support image embedding.
    async fn embed_image(&self, _image_data: &[u8]) -> Result<Vec<f64>> {
        Err(ChainError::EmbeddingError(
            "Image embedding is not supported by this model".into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Fake / test helpers
// ---------------------------------------------------------------------------

/// A fake embedding model that produces deterministic hash-based vectors.
///
/// The same input text always yields the same output vector (within a single
/// process), making it suitable for tests that need reproducible embeddings
/// without calling a real API.
///
/// # Example
/// ```ignore
/// use langchain_core::embeddings::FakeEmbeddings;
/// use langchain_core::embeddings::Embeddings;
///
/// let fake = FakeEmbeddings::new(64);
/// let vec = fake.embed_query("hello").await?;
/// assert_eq!(vec.len(), 64);
/// ```
pub struct FakeEmbeddings {
    dimension: usize,
}

impl FakeEmbeddings {
    /// Create a new `FakeEmbeddings` that produces vectors of the given dimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

/// Simple DJB-style hash for deterministic embedding generation.
fn hash_str(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}

/// Produce a deterministic embedding vector from text and dimension.
fn make_embedding(text: &str, dimension: usize) -> Vec<f64> {
    let h = hash_str(text);
    (0..dimension)
        .map(|i| {
            let v = hash_str(&format!("{}-{}", h, i));
            (v % 1000) as f64 / 1000.0
        })
        .collect()
}

#[async_trait]
impl Embeddings for FakeEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f64>>> {
        Ok(texts
            .iter()
            .map(|t| make_embedding(t, self.dimension))
            .collect())
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f64>> {
        Ok(make_embedding(text, self.dimension))
    }
}

/// A fully deterministic fake embedding model.
///
/// Unlike [`FakeEmbeddings`] (which uses a DJB hash), this uses a simple
/// character-sum scheme so that the output is trivially predictable and
/// stable across process restarts. This makes it ideal for golden-file
/// tests or cross-language consistency checks.
///
/// Each dimension `i` of the embedding for text `t` is computed as:
/// `((sum of byte values of t) + i) % 1000) / 1000.0`
///
/// # Example
/// ```ignore
/// use langchain_core::embeddings::DeterministicFakeEmbedding;
/// use langchain_core::embeddings::Embeddings;
///
/// let emb = DeterministicFakeEmbedding::new(32);
/// let v = emb.embed_query("abc").await?;
/// assert_eq!(v.len(), 32);
/// ```
pub struct DeterministicFakeEmbedding {
    dimension: usize,
}

impl DeterministicFakeEmbedding {
    /// Create a new `DeterministicFakeEmbedding` with the given vector dimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

fn make_deterministic_embedding(text: &str, dimension: usize) -> Vec<f64> {
    let byte_sum: u64 = text.bytes().map(|b| b as u64).sum();
    (0..dimension)
        .map(|i| ((byte_sum + i as u64) % 1000) as f64 / 1000.0)
        .collect()
}

#[async_trait]
impl Embeddings for DeterministicFakeEmbedding {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f64>>> {
        Ok(texts
            .iter()
            .map(|t| make_deterministic_embedding(t, self.dimension))
            .collect())
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f64>> {
        Ok(make_deterministic_embedding(text, self.dimension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_embeddings_dimension() {
        let fake = FakeEmbeddings::new(64);
        let vec = fake.embed_query("hello").await.unwrap();
        assert_eq!(vec.len(), 64);
    }

    #[tokio::test]
    async fn test_fake_embeddings_deterministic() {
        let fake = FakeEmbeddings::new(32);
        let v1 = fake.embed_query("test").await.unwrap();
        let v2 = fake.embed_query("test").await.unwrap();
        assert_eq!(v1, v2);
    }

    #[tokio::test]
    async fn test_fake_embeddings_different_inputs() {
        let fake = FakeEmbeddings::new(32);
        let v1 = fake.embed_query("hello").await.unwrap();
        let v2 = fake.embed_query("world").await.unwrap();
        assert_ne!(v1, v2);
    }

    #[tokio::test]
    async fn test_fake_embeddings_documents() {
        let fake = FakeEmbeddings::new(16);
        let docs = vec!["foo".into(), "bar".into()];
        let vecs = fake.embed_documents(&docs).await.unwrap();
        assert_eq!(vecs.len(), 2);
        assert_eq!(vecs[0].len(), 16);
        assert_ne!(vecs[0], vecs[1]);
    }

    #[tokio::test]
    async fn test_deterministic_fake_embedding() {
        let emb = DeterministicFakeEmbedding::new(32);
        let v1 = emb.embed_query("abc").await.unwrap();
        let v2 = emb.embed_query("abc").await.unwrap();
        assert_eq!(v1, v2);
        assert_eq!(v1.len(), 32);
    }

    #[tokio::test]
    async fn test_default_embed_image_unsupported() {
        let fake = FakeEmbeddings::new(8);
        let result = fake.embed_image(&[0u8, 1, 2]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fake_embeddings_empty_documents() {
        let fake = FakeEmbeddings::new(8);
        let vecs = fake.embed_documents(&[]).await.unwrap();
        assert!(vecs.is_empty());
    }

    #[tokio::test]
    async fn test_fake_embeddings_various_dimensions() {
        for dim in [1, 2, 100, 512] {
            let fake = FakeEmbeddings::new(dim);
            let vec = fake.embed_query("x").await.unwrap();
            assert_eq!(vec.len(), dim);
        }
    }

    #[tokio::test]
    async fn test_deterministic_fake_embedding_documents() {
        let emb = DeterministicFakeEmbedding::new(16);
        let docs = vec!["alpha".into(), "beta".into()];
        let vecs = emb.embed_documents(&docs).await.unwrap();
        assert_eq!(vecs.len(), 2);
        assert_eq!(vecs[0].len(), 16);
    }

    #[tokio::test]
    async fn test_deterministic_fake_embedding_cross_process_stability() {
        let emb = DeterministicFakeEmbedding::new(4);
        let v = emb.embed_query("abc").await.unwrap();
        let byte_sum = 97u64 + 98 + 99;
        let expected: Vec<f64> = (0..4)
            .map(|i| ((byte_sum + i as u64) % 1000) as f64 / 1000.0)
            .collect();
        assert_eq!(v, expected);
    }

    #[tokio::test]
    async fn test_deterministic_fake_embedding_empty_documents() {
        let emb = DeterministicFakeEmbedding::new(8);
        let vecs = emb.embed_documents(&[]).await.unwrap();
        assert!(vecs.is_empty());
    }

    #[test]
    fn test_fake_embeddings_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeEmbeddings>();
        assert_sync::<FakeEmbeddings>();
    }

    #[test]
    fn test_deterministic_fake_embedding_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<DeterministicFakeEmbedding>();
        assert_sync::<DeterministicFakeEmbedding>();
    }

    #[test]
    fn test_hash_str_consistency() {
        let h1 = hash_str("hello");
        let h2 = hash_str("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_str_different() {
        let h1 = hash_str("hello");
        let h2 = hash_str("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_str_empty() {
        let h = hash_str("");
        assert_eq!(h, 5381);
    }
}
