//! Mistral AI embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const MISTRAL_BASE_URL: &str = "https://api.mistral.ai/v1";

pub struct MistralEmbeddings {
    api_key: String,
    model: String,
    client: Client,
}

impl std::fmt::Debug for MistralEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MistralEmbeddings")
            .field("model", &self.model)
            .finish()
    }
}

#[derive(Serialize)]
struct MistralEmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct MistralEmbedResponse {
    id: String,
    object: String,
    data: Vec<MistralEmbedData>,
    model: String,
    usage: MistralEmbedUsage,
}

#[derive(Deserialize)]
struct MistralEmbedData {
    object: String,
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Deserialize)]
struct MistralEmbedUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

impl MistralEmbeddings {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "mistral-embed".to_string(),
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
            .post(format!("{}/embeddings", MISTRAL_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&MistralEmbedRequest {
                model: self.model.clone(),
                input: texts.to_vec(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Mistral embedding request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Mistral API error ({}): {}",
                status, body
            )));
        }

        let result: MistralEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Mistral embedding: {}", e))
        })?;

        let mut data = result.data;
        data.sort_by_key(|d| d.index);
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

#[async_trait]
impl Embeddings for MistralEmbeddings {
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
