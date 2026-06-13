//! spaCy embedding provider.
//!
//! Uses spaCy's built-in word vectors and transformer model pipelines
//! to generate text embeddings locally.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// spaCy embedding model.
///
/// spaCy provides statistical NLP models with word vectors that can
/// produce dense text representations. This stub returns deterministic
/// size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::spacy::SpacyEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = SpacyEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct SpacyEmbeddings {
    _private: (),
}

impl SpacyEmbeddings {
    /// Creates a new `SpacyEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for SpacyEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for SpacyEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(97).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(97).wrapping_add(b as u64));
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
