//! DeepInfra BGE embedding provider (stub).
//!
//! DeepInfra-hosted BGE embedding models. This stub returns deterministic
//! size-4 vectors.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// DeepInfra BGE embedding model (providers stub).
///
/// DeepInfra-hosted BGE text embedding models.
/// This stub returns deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::deepinfra_bge::DeepInfraBGEEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = DeepInfraBGEEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct DeepInfraBGEEmbeddings {
    _private: (),
}

impl DeepInfraBGEEmbeddings {
    /// Creates a new `DeepInfraBGEEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for DeepInfraBGEEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for DeepInfraBGEEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(191).wrapping_add(b as u64));
                vec![
                    (h % 100) as f32 / 100.0,
                    ((h + 1) % 100) as f32 / 100.0,
                    ((h + 2) % 100) as f32 / 100.0,
                    ((h + 3) % 100) as f32 / 100.0,
                ]
            })
            .collect())
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let h = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(191).wrapping_add(b as u64));
        Ok(vec![
            (h % 100) as f32 / 100.0,
            ((h + 1) % 100) as f32 / 100.0,
            ((h + 2) % 100) as f32 / 100.0,
            ((h + 3) % 100) as f32 / 100.0,
        ])
    }

    fn embedding_dimension(&self) -> usize {
        4
    }
}
