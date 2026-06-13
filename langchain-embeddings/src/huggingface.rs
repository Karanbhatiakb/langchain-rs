//! HuggingFace embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

const HF_INFERENCE_URL: &str = "https://api-inference.huggingface.co/models";

pub struct HuggingFaceEmbeddings {
    model_name: String,
    api_url: String,
    api_key: String,
    client: Client,
}

impl std::fmt::Debug for HuggingFaceEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HuggingFaceEmbeddings")
            .field("model_name", &self.model_name)
            .field("api_url", &self.api_url)
            .finish()
    }
}

#[derive(Serialize)]
struct HFEmbeddingRequest {
    inputs: Vec<String>,
    options: HFOptions,
}

#[derive(Serialize)]
struct HFOptions {
    wait_for_model: bool,
}

#[derive(Deserialize)]
struct HFEmbeddingResponse {
    #[serde(default)]
    embeddings: Vec<Vec<f32>>,
}

impl HuggingFaceEmbeddings {
    pub fn new(model_name: impl Into<String>, api_key: impl Into<String>) -> Self {
        let name = model_name.into();
        Self {
            api_url: format!("{}/{}", HF_INFERENCE_URL, name),
            model_name: name,
            api_key: api_key.into(),
            client: Client::new(),
        }
    }

    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&HFEmbeddingRequest {
                inputs: texts.to_vec(),
                options: HFOptions {
                    wait_for_model: true,
                },
            })
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("HuggingFace request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "HuggingFace API error ({}): {}",
                status, body
            )));
        }

        let result: Vec<Vec<f32>> = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse HF embedding: {}", e))
        })?;

        Ok(result)
    }
}

#[async_trait]
impl Embeddings for HuggingFaceEmbeddings {
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
        768
    }
}
