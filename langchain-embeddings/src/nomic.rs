//! Nomic embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const NOMIC_BASE_URL: &str = "https://api-atlas.nomic.ai/v1";

pub struct NomicEmbeddings {
    api_key: String,
    model: String,
    client: Client,
}

impl std::fmt::Debug for NomicEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NomicEmbeddings")
            .field("model", &self.model)
            .finish()
    }
}

#[derive(Serialize)]
struct NomicEmbedRequest {
    model: String,
    texts: Vec<String>,
}

#[derive(Deserialize)]
struct NomicEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

impl NomicEmbeddings {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "nomic-embed-text-v1.5".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .post(format!("{}/embedding/text", NOMIC_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&NomicEmbedRequest {
                model: self.model.clone(),
                texts: texts.to_vec(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Nomic request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Nomic API error ({}): {}",
                status, body
            )));
        }

        let result: NomicEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Nomic embedding: {}", e))
        })?;

        Ok(result.embeddings)
    }
}

#[async_trait]
impl Embeddings for NomicEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed_batch(texts).await
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::EmbeddingError("No embedding returned".to_string()))
    }

    fn embedding_dimension(&self) -> usize {
        768
    }
}
