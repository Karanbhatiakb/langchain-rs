//! Core [`Embeddings`] trait for converting text to vector embeddings.

use async_trait::async_trait;
use langchain_core::errors::Result;

/// Trait for embedding models that convert text into dense vector
/// representations.
///
/// # Type parameters
/// Implementors define `embed_documents` for batch embedding and
/// `embed_query` for single-query embedding.
#[async_trait]
pub trait Embeddings: Send + Sync {
    /// Embeds a batch of documents, returning a vector of embeddings.
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    /// Embeds a single query string into a vector.
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>>;

    /// Returns the dimensionality of the embedding vectors.
    fn embedding_dimension(&self) -> usize {
        1536
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEmbeddings;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_embeddings_trait_object() {
        let e: Arc<dyn Embeddings> = Arc::new(FakeEmbeddings::new(8));
        let result = e.embed_query("test").await.unwrap();
        assert_eq!(result.len(), 8);
    }

    #[tokio::test]
    async fn test_embedding_dimension_default() {
        struct DummyEmbeddings;
        #[async_trait]
        impl Embeddings for DummyEmbeddings {
            async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
                Ok(vec![vec![0.0; 1536]; texts.len()])
            }
            async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
                Ok(vec![0.0; 1536])
            }
        }
        assert_eq!(DummyEmbeddings.embedding_dimension(), 1536);
    }

    #[tokio::test]
    async fn test_embeddings_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeEmbeddings>();
        assert_sync::<FakeEmbeddings>();
    }
}
