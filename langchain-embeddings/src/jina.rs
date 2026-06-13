//! Jina AI embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const JINA_BASE_URL: &str = "https://api.jina.ai/v1";

pub struct JinaEmbeddings {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
}

impl std::fmt::Debug for JinaEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JinaEmbeddings")
            .field("model", &self.model)
            .finish()
    }
}

#[derive(Serialize)]
struct JinaEmbedRequest {
    model: String,
    input: Vec<String>,
    encoding_type: String,
}

#[derive(Deserialize)]
struct JinaEmbedResponse {
    data: Vec<JinaEmbedData>,
    model: String,
    usage: JinaEmbedUsage,
}

#[derive(Deserialize)]
struct JinaEmbedData {
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Deserialize)]
struct JinaEmbedUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

impl JinaEmbeddings {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "jina-embeddings-v3".to_string(),
            base_url: JINA_BASE_URL.to_string(),
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
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&JinaEmbedRequest {
                model: self.model.clone(),
                input: texts.to_vec(),
                encoding_type: "float".to_string(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Jina request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Jina API error ({}): {}",
                status, body
            )));
        }

        let result: JinaEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Jina embedding: {}", e))
        })?;

        let mut data = result.data;
        data.sort_by_key(|d| d.index);
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

#[async_trait]
impl Embeddings for JinaEmbeddings {
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
