//! Sentence-Transformer embedding provider.
//!
//! Uses HuggingFace's `sentence-transformers` models locally to produce
//! sentence and paragraph embeddings.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// Sentence-Transformer local embedding model.
///
/// Sentence-Transformer models map sentences and paragraphs to dense
/// vectors optimised for semantic similarity. This stub returns
/// deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::sentence_transformer::SentenceTransformerEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = SentenceTransformerEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct SentenceTransformerEmbeddings {
    _private: (),
}

impl SentenceTransformerEmbeddings {
    /// Creates a new `SentenceTransformerEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for SentenceTransformerEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for SentenceTransformerEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(23).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(23).wrapping_add(b as u64));
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
