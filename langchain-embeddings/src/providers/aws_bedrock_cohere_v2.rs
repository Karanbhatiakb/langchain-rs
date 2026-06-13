//! AWS Bedrock Cohere v2 embedding provider.
//!
//! Cohere v2 embeddings served through AWS Bedrock.

use async_trait::async_trait;
use langchain_core::errors::Result;

use crate::traits::Embeddings;

/// AWS Bedrock Cohere v2 embedding model.
///
/// This stub returns deterministic size-4 vectors.
///
/// # Example
/// ```ignore
/// use langchain_embeddings::providers::aws_bedrock_cohere_v2::AwsBedrockCohereV2Embeddings;
/// use langchain_core::traits::Embeddings;
///
/// let emb = AwsBedrockCohereV2Embeddings::new();
/// let vec = emb.embed_query("hello").await.unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct AwsBedrockCohereV2Embeddings {
    _private: (),
}

impl AwsBedrockCohereV2Embeddings {
    /// Creates a new `AwsBedrockCohereV2Embeddings` instance.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for AwsBedrockCohereV2Embeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Embeddings for AwsBedrockCohereV2Embeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| {
                let h = t
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(181).wrapping_add(b as u64));
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
            .fold(0u64, |acc, b| acc.wrapping_mul(181).wrapping_add(b as u64));
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
