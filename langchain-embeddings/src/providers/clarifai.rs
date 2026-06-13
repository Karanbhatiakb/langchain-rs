//! Clarifai embedding provider.
//!
//! Clarifai offers a platform for building and deploying AI models,
//! including text and multimodal embeddings.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// Clarifai embedding model.
///
/// Clarifai provides pre-trained and custom models for generating text
/// and multimodal embeddings. This stub returns deterministic size-4
/// vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::clarifai::ClarifaiEmbeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = ClarifaiEmbeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ClarifaiEmbeddings {
    _private: (),
}

impl ClarifaiEmbeddings {
    /// Creates a new `ClarifaiEmbeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for ClarifaiEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for ClarifaiEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(53).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(53).wrapping_add(b as u64));
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
