//! HuggingFace Inference API embedding provider.
//!
//! Uses HuggingFace's hosted Inference API to generate text embeddings
//! from the Hub's model repository.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// HuggingFace Inference API embedding model.
///
/// The HuggingFace Inference API provides access to thousands of
/// embedding models hosted on the HuggingFace Hub. This stub returns
/// deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::huggingface_inference::HuggingFaceInferenceEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = HuggingFaceInferenceEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct HuggingFaceInferenceEmbeddings {
    _private: (),
}

impl HuggingFaceInferenceEmbeddings {
    /// Creates a new `HuggingFaceInferenceEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for HuggingFaceInferenceEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for HuggingFaceInferenceEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(73).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(73).wrapping_add(b as u64));
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
