//! John Snow Labs embedding provider.
//!
//! John Snow Labs provides healthcare and NLP-focused embedding models
//! via the Spark NLP ecosystem.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// John Snow Labs embedding model.
///
/// John Snow Labs offers specialised embedding models for healthcare
/// and biomedical NLP tasks. This stub returns deterministic size-4
/// vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::johnsnowlabs::JohnSnowLabsEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = JohnSnowLabsEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct JohnSnowLabsEmbeddings {
    _private: (),
}

impl JohnSnowLabsEmbeddings {
    /// Creates a new `JohnSnowLabsEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for JohnSnowLabsEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for JohnSnowLabsEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(79).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(79).wrapping_add(b as u64));
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
