//! Alibaba DashScope embedding provider (stub).
//!
//! DashScope (Alibaba Cloud) embedding models via their API. This stub returns
//! deterministic size-4 vectors.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// Alibaba DashScope embedding model (providers stub).
///
/// DashScope provides embedding models like `text-embedding-v1`.
/// This stub returns deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::alibaba_dashscope::AlibabaDashScopeEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = AlibabaDashScopeEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct AlibabaDashScopeEmbeddings {
    _private: (),
}

impl AlibabaDashScopeEmbeddings {
    /// Creates a new `AlibabaDashScopeEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for AlibabaDashScopeEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for AlibabaDashScopeEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(61).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(61).wrapping_add(b as u64));
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
