//! Voyage AI embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const VOYAGE_BASE_URL: &str = "https://api.voyageai.com/v1";
const DEFAULT_MODEL: &str = "voyage-2";

pub struct VoyageEmbeddings {
    api_key: String,
    model: String,
    client: Client,
}

impl std::fmt::Debug for VoyageEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoyageEmbeddings")
            .field("model", &self.model)
            .finish()
    }
}

#[derive(Serialize)]
struct VoyageEmbedRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct VoyageEmbedResponse {
    data: Vec<VoyageEmbedData>,
}

#[derive(Deserialize)]
struct VoyageEmbedData {
    embedding: Vec<f32>,
    index: u32,
}

impl VoyageEmbeddings {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: DEFAULT_MODEL.to_string(),
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
            .post(format!("{}/embeddings", VOYAGE_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&VoyageEmbedRequest {
                input: texts.to_vec(),
                model: self.model.clone(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Voyage request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Voyage API error ({}): {}",
                status, body
            )));
        }

        let result: VoyageEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Voyage embedding: {}", e))
        })?;

        let mut data = result.data;
        data.sort_by_key(|d| d.index);
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

#[async_trait]
impl Embeddings for VoyageEmbeddings {
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
        1024
    }
}
