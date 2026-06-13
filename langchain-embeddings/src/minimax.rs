//! MiniMax embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const MINIMAX_BASE_URL: &str = "https://api.minimax.chat/v1";

pub struct MiniMaxEmbeddings {
    api_key: String,
    group_id: String,
    model: String,
    client: Client,
}

impl std::fmt::Debug for MiniMaxEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiniMaxEmbeddings")
            .field("model", &self.model)
            .finish()
    }
}

#[derive(Serialize)]
struct MiniMaxEmbedRequest {
    model: String,
    texts: Vec<String>,
}

#[derive(Deserialize)]
struct MiniMaxEmbedResponse {
    vectors: Vec<Vec<f32>>,
}

impl MiniMaxEmbeddings {
    pub fn new(api_key: impl Into<String>, group_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            group_id: group_id.into(),
            model: "embo-01".to_string(),
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
            .post(format!(
                "{}/embeddings?GroupId={}",
                MINIMAX_BASE_URL, self.group_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&MiniMaxEmbedRequest {
                model: self.model.clone(),
                texts: texts.to_vec(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("MiniMax request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "MiniMax API error ({}): {}",
                status, body
            )));
        }

        let result: MiniMaxEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse MiniMax embedding: {}", e))
        })?;

        Ok(result.vectors)
    }
}

#[async_trait]
impl Embeddings for MiniMaxEmbeddings {
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
        2560
    }
}
