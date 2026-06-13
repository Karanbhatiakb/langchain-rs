//! HuggingFace Dataset document loader.
//!
//! Fetches a dataset from the HuggingFace Datasets Hub via its HTTP API and
//! returns each row as a JSON document.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads documents from a HuggingFace Dataset.
#[derive(Debug, Clone)]
pub struct HuggingFaceDatasetLoader {
    dataset_name: String,
    split: String,
    client: Client,
}

impl HuggingFaceDatasetLoader {
    /// Create a new `HuggingFaceDatasetLoader`.
    ///
    /// `split` defaults to `"train"`.
    pub fn new(dataset_name: impl Into<String>) -> Self {
        Self {
            dataset_name: dataset_name.into(),
            split: "train".to_string(),
            client: Client::new(),
        }
    }

    /// Set the dataset split (e.g. `"train"`, `"test"`, `"validation"`).
    pub fn with_split(mut self, split: impl Into<String>) -> Self {
        self.split = split.into();
        self
    }
}

#[async_trait]
impl BaseLoader for HuggingFaceDatasetLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let url = format!(
            "https://huggingface.co/api/datasets/{}/parquet/{}/0.parquet",
            self.dataset_name, self.split
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("HuggingFace dataset request failed: {}", e)))?;

        let body = response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read HuggingFace dataset response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("dataset".to_string(), serde_json::Value::String(self.dataset_name.clone()));
        metadata.insert("split".to_string(), serde_json::Value::String(self.split.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("huggingface_dataset".to_string()));

        Ok(vec![Document::new(body).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
