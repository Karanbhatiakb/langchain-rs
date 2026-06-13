//! LlamaCpp embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const LLAMACPP_DEFAULT_URL: &str = "http://localhost:8080";

pub struct LlamaCppEmbeddings {
    base_url: String,
    client: Client,
}

impl std::fmt::Debug for LlamaCppEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaCppEmbeddings")
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct LlamaCppEmbedRequest {
    content: String,
}

#[derive(Deserialize)]
struct LlamaCppEmbedResponse {
    embedding: Vec<f32>,
}

impl LlamaCppEmbeddings {
    pub fn new() -> Self {
        Self {
            base_url: LLAMACPP_DEFAULT_URL.to_string(),
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
            .post(format!("{}/embedding", self.base_url))
            .header("Content-Type", "application/json")
            .json(&LlamaCppEmbedRequest {
                content: text.to_string(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("llama.cpp request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "llama.cpp API error ({}): {}",
                status, body
            )));
        }

        let result: LlamaCppEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse llama.cpp embedding: {}", e))
        })?;

        Ok(result.embedding)
    }
}

#[async_trait]
impl Embeddings for LlamaCppEmbeddings {
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
