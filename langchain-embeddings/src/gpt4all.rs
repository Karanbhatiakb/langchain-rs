//! GPT4All embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const GPT4ALL_DEFAULT_URL: &str = "http://localhost:8080";

pub struct GPT4AllEmbeddings {
    base_url: String,
    client: Client,
}

impl std::fmt::Debug for GPT4AllEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GPT4AllEmbeddings")
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct GPT4AllEmbedRequest {
    content: String,
}

#[derive(Deserialize)]
struct GPT4AllEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

impl GPT4AllEmbeddings {
    pub fn new() -> Self {
        Self {
            base_url: GPT4ALL_DEFAULT_URL.to_string(),
            client: Client::new(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let response = self
            .client
            .post(format!("{}/api/v1/embeddings", self.base_url))
            .header("Content-Type", "application/json")
            .json(&GPT4AllEmbedRequest {
                content: text.to_string(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("GPT4All request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "GPT4All API error ({}): {}",
                status, body
            )));
        }

        let result: GPT4AllEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse GPT4All embedding: {}", e))
        })?;

        result
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::EmbeddingError("No embedding returned".to_string()))
    }
}

#[async_trait]
impl Embeddings for GPT4AllEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        self.embed(text).await
    }

    fn embedding_dimension(&self) -> usize {
        768
    }
}
