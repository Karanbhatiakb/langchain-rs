//! Self-hosted Llama embedding provider.
//!
//! Runs Llama-family models (e.g., via llama.cpp or vLLM) on your own
//! infrastructure for embedding generation.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// Self-hosted Llama embedding model.
///
/// Connects to a self-hosted endpoint serving a Llama-family model
/// for text embeddings. This stub returns deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::self_hosted_llama::SelfHostedLlamaEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = SelfHostedLlamaEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct SelfHostedLlamaEmbeddings {
    _private: (),
}

impl SelfHostedLlamaEmbeddings {
    /// Creates a new `SelfHostedLlamaEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for SelfHostedLlamaEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for SelfHostedLlamaEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u64));
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
