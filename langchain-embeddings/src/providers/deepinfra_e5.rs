//! DeepInfra E5 embedding provider (stub).
//!
//! DeepInfra-hosted E5 embedding models. This stub returns deterministic
//! size-4 vectors.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// DeepInfra E5 embedding model (providers stub).
///
/// DeepInfra-hosted E5 text embedding models.
/// This stub returns deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::deepinfra_e5::DeepInfraE5Embeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = DeepInfraE5Embeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct DeepInfraE5Embeddings {
    _private: (),
}

impl DeepInfraE5Embeddings {
    /// Creates a new `DeepInfraE5Embeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for DeepInfraE5Embeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for DeepInfraE5Embeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(193).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(193).wrapping_add(b as u64));
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
