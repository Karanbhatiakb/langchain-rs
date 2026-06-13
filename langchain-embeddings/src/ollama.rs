//! Ollama local embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;

use crate::traits::Embeddings;

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

pub struct OllamaEmbeddings {
    model: String,
    base_url: String,
    client: Client,
}

impl std::fmt::Debug for OllamaEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaEmbeddings")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

impl OllamaEmbeddings {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: OLLAMA_BASE_URL.to_string(),
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
            .post(format!("{}/api/embeddings", self.base_url))
            .header("Content-Type", "application/json")
            .json(&OllamaEmbedRequest {
                model: self.model.clone(),
                prompt: text.to_string(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Ollama embedding request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Ollama API error ({}): {}",
                status, body
            )));
        }

        let result: OllamaEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Ollama embedding: {}", e))
        })?;

        Ok(result.embedding)
    }
}

#[async_trait]
impl Embeddings for OllamaEmbeddings {
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
        4096
    }
}
