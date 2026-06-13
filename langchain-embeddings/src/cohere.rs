//! Cohere embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const COHERE_BASE_URL: &str = "https://api.cohere.com/v2";

pub struct CohereEmbeddings {
    api_key: String,
    model: String,
    client: Client,
}

impl std::fmt::Debug for CohereEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CohereEmbeddings")
            .field("model", &self.model)
            .finish()
    }
}

#[derive(Serialize)]
struct CohereEmbedRequest {
    texts: Vec<String>,
    model: String,
    #[serde(rename = "input_type")]
    input_type: String,
}

#[derive(Deserialize)]
struct CohereEmbedResponse {
    embeddings: Vec<Vec<f32>>,
    meta: Option<CohereMeta>,
}

#[derive(Deserialize)]
struct CohereMeta {
    api_version: Option<CohereApiVersion>,
}

#[derive(Deserialize)]
struct CohereApiVersion {
    version: String,
}

impl CohereEmbeddings {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "embed-english-v3.0".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    async fn embed_batch(&self, texts: &[String], input_type: &str) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .post(format!("{}/embed", COHERE_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&CohereEmbedRequest {
                texts: texts.to_vec(),
                model: self.model.clone(),
                input_type: input_type.to_string(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Cohere request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Cohere API error ({}): {}",
                status, body
            )));
        }

        let result: CohereEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Cohere embedding: {}", e))
        })?;

        Ok(result.embeddings)
    }
}

#[async_trait]
impl Embeddings for CohereEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed_batch(texts, "search_document").await
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self
            .embed_batch(&[text.to_string()], "search_query")
            .await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::EmbeddingError("No embedding returned".to_string()))
    }

    fn embedding_dimension(&self) -> usize {
        1024
    }
}
