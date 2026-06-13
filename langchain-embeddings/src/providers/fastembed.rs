//! FastEmbed embedding provider.
//!
//! FastEmbed provides fast, local embedding inference using quantised
//! models via the `fastembed` Rust crate.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// FastEmbed local embedding model.
///
/// FastEmbed runs embedding models locally using ONNX Runtime with
/// quantised weights for fast CPU inference. This stub returns
/// deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::fastembed::FastEmbedEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = FastEmbedEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct FastEmbedEmbeddings {
    _private: (),
}

impl FastEmbedEmbeddings {
    /// Creates a new `FastEmbedEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for FastEmbedEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for FastEmbedEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(19).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(19).wrapping_add(b as u64));
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
