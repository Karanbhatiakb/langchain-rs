//! Cohere Embed English v3 provider (stub).
//!
//! Cohere embed-english-v3 model. This stub returns deterministic
//! size-4 vectors.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// Cohere Embed English v3 model (providers stub).
///
/// Cohere embed-english-v3 embedding model.
/// This stub returns deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::cohere_embed_english_v3::CohereEmbedEnglishV3Embeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = CohereEmbedEnglishV3Embeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct CohereEmbedEnglishV3Embeddings {
    _private: (),
}

impl CohereEmbedEnglishV3Embeddings {
    /// Creates a new `CohereEmbedEnglishV3Embeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for CohereEmbedEnglishV3Embeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for CohereEmbedEnglishV3Embeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(157).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(157).wrapping_add(b as u64));
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
