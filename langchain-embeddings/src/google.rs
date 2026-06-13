//! Google embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

pub struct GoogleEmbeddings {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
}

impl std::fmt::Debug for GoogleEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleEmbeddings")
            .field("model", &self.model)
            .finish()
    }
}

#[derive(Serialize)]
struct GoogleEmbedRequest {
    model: String,
    content: GoogleContent,
}

#[derive(Serialize)]
struct GoogleContent {
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Deserialize)]
struct GoogleEmbedResponse {
    embedding: GoogleEmbeddingValue,
}

#[derive(Deserialize)]
struct GoogleEmbeddingValue {
    values: Vec<f32>,
}

impl GoogleEmbeddings {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "text-embedding-004".to_string(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!(
            "{}/v1beta/models/{}:embedContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&GoogleEmbedRequest {
                model: format!("models/{}", self.model),
                content: GoogleContent {
                    parts: vec![GooglePart {
                        text: text.to_string(),
                    }],
                },
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Google embedding request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Google API error ({}): {}",
                status, body
            )));
        }

        let result: GoogleEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Google embedding: {}", e))
        })?;

        Ok(result.embedding.values)
    }
}

#[async_trait]
impl Embeddings for GoogleEmbeddings {
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
