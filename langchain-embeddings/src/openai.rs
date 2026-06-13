//! OpenAI embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::warn;

use crate::traits::Embeddings;

const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "text-embedding-3-small";

pub struct OpenAIEmbeddings {
    api_key: String,
    model: String,
    chunk_size: usize,
    client: Client,
    max_retries: u32,
    timeout: Duration,
    dimensions: Option<u32>,
}

impl std::fmt::Debug for OpenAIEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIEmbeddings")
            .field("model", &self.model)
            .field("chunk_size", &self.chunk_size)
            .finish()
    }
}

#[derive(Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    usage: EmbeddingUsage,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Deserialize)]
struct EmbeddingUsage {
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

impl OpenAIEmbeddings {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: DEFAULT_MODEL.to_string(),
            chunk_size: 1000,
            client: Client::new(),
            max_retries: 3,
            timeout: Duration::from_secs(60),
            dimensions: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    pub fn with_dimensions(mut self, dimensions: u32) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let url = format!("{}/embeddings", OPENAI_BASE_URL);
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            let request = EmbeddingRequest {
                input: texts.to_vec(),
                model: self.model.clone(),
                dimensions: self.dimensions,
            };

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(self.timeout)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let result: EmbeddingResponse = resp.json().await.map_err(|e| {
                            ChainError::ParserError(format!("Failed to parse embedding: {}", e))
                        })?;
                let mut data = result.data;
                data.sort_by_key(|d| d.index);
                return Ok(data.into_iter().map(|d| d.embedding).collect());
                    }

                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();

                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(ChainError::RateLimitError("Rate limited by OpenAI API".to_string()));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        warn!("Rate limited, retrying in {:?}", wait);
                        tokio::time::sleep(wait).await;
                        continue;
                    }

                    return Err(ChainError::LLMError(format!(
                        "OpenAI embedding error ({}): {}",
                        status, body
                    )));
                }
                Err(e) => {
                    last_error = Some(ChainError::LLMError(format!(
                        "Embedding request failed: {}",
                        e
                    )));
                    if attempt < self.max_retries - 1 {
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        tokio::time::sleep(wait).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ChainError::LLMError("Max retries exceeded for embeddings".to_string())
        }))
    }
}

#[async_trait]
impl Embeddings for OpenAIEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings = Vec::new();
        for chunk in texts.chunks(self.chunk_size) {
            let embeddings = self.embed_batch(chunk).await?;
            all_embeddings.extend(embeddings);
        }
        Ok(all_embeddings)
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::EmbeddingError("No embedding returned".to_string()))
    }

    fn embedding_dimension(&self) -> usize {
        match self.model.as_str() {
            "text-embedding-3-large" => 3072,
            "text-embedding-3-small" => 1536,
            "text-embedding-ada-002" => 1536,
            _ => 1536,
        }
    }
}
