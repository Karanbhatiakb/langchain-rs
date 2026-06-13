//! Azure OpenAI embedding model provider.

use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::Embeddings;

pub struct AzureEmbeddings {
    api_key: String,
    resource: String,
    deployment: String,
    api_version: String,
    client: Client,
}

impl std::fmt::Debug for AzureEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureEmbeddings")
            .field("resource", &self.resource)
            .field("deployment", &self.deployment)
            .finish()
    }
}

#[derive(Serialize)]
struct AzureEmbedRequest {
    input: Vec<String>,
}

#[derive(Deserialize)]
struct AzureEmbedResponse {
    data: Vec<AzureEmbedData>,
}

#[derive(Deserialize)]
struct AzureEmbedData {
    embedding: Vec<f32>,
    index: u32,
}

impl AzureEmbeddings {
    pub fn new(api_key: impl Into<String>, resource: impl Into<String>, deployment: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            resource: resource.into(),
            deployment: deployment.into(),
            api_version: "2023-05-15".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    fn build_url(&self) -> String {
        format!(
            "https://{}.openai.azure.com/openai/deployments/{}/embeddings?api-version={}",
            self.resource, self.deployment, self.api_version
        )
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .post(self.build_url())
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&AzureEmbedRequest {
                input: texts.to_vec(),
            })
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Azure request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Azure API error ({}): {}",
                status, body
            )));
        }

        let result: AzureEmbedResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Azure embedding: {}", e))
        })?;

        let mut data = result.data;
        data.sort_by_key(|d| d.index);
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

#[async_trait]
impl Embeddings for AzureEmbeddings {
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
        1536
    }
}
