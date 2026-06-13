//! Fake embedding implementation for testing and prototyping.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// A fake embedding model that produces deterministic hash-based embeddings.
///
/// Useful for testing pipelines without calling a real embedding API.
pub struct FakeEmbeddings {
    dimension: usize,
}

impl FakeEmbeddings {
    /// Creates a new `FakeEmbeddings` with the given vector dimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

/// Computes a simple hash of a string.
fn hash_str(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}

/// Produces a deterministic embedding vector from a text string.
fn make_embedding(text: &str, dimension: usize) -> Vec<f32> {
    let h = hash_str(text);
    (0..dimension)
        .map(|i| {
            let v = hash_str(&format!("{}-{}", h, i));
            (v % 1000) as f32 / 1000.0
        })
        .collect()
}

#[async_trait]
impl Embeddings for FakeEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| make_embedding(t, self.dimension))
            .collect())
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        Ok(make_embedding(text, self.dimension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_embeddings_embed_documents() {
        let embeddings = FakeEmbeddings::new(4);
        let texts = vec!["hello".to_string(), "world".to_string()];
        let result = embeddings.embed_documents(&texts).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 4);
    }

    #[tokio::test]
    async fn test_fake_embeddings_embed_query() {
        let embeddings = FakeEmbeddings::new(8);
        let result = embeddings.embed_query("test query").await.unwrap();
        assert_eq!(result.len(), 8);
    }

    #[tokio::test]
    async fn test_fake_embeddings_deterministic() {
        let embeddings = FakeEmbeddings::new(4);
        let result1 = embeddings.embed_query("hello").await.unwrap();
        let result2 = embeddings.embed_query("hello").await.unwrap();
        assert_eq!(result1, result2);
    }

    #[tokio::test]
    async fn test_fake_embeddings_different_texts_different_vectors() {
        let embeddings = FakeEmbeddings::new(4);
        let result1 = embeddings.embed_query("hello").await.unwrap();
        let result2 = embeddings.embed_query("world").await.unwrap();
        assert_ne!(result1, result2);
    }

    #[tokio::test]
    async fn test_fake_embeddings_dimension() {
        let embeddings = FakeEmbeddings::new(16);
        let result = embeddings.embed_query("test").await.unwrap();
        assert_eq!(result.len(), 16);
    }

    #[tokio::test]
    async fn test_fake_embeddings_empty_documents() {
        let embeddings = FakeEmbeddings::new(4);
        let result = embeddings.embed_documents(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_fake_embeddings_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeEmbeddings>();
        assert_sync::<FakeEmbeddings>();
    }
}
