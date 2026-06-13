//! Ollama embedding provider (providers stub).
//!
//! Ollama provides local embedding inference via a REST API. A full
//! implementation lives at the crate root.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// Ollama embedding model (providers stub).
///
/// Ollama runs embedding models locally as a background service with a
/// simple HTTP API. This stub returns deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::ollama::OllamaEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = OllamaEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct OllamaEmbeddings {
    _private: (),
}

impl OllamaEmbeddings {
    /// Creates a new `OllamaEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for OllamaEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for OllamaEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(89).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(89).wrapping_add(b as u64));
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
