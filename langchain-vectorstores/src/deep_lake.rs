//! Deep Lake vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct DeepLakeVectorStore {
    dataset_path: String,
    api_key: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
    overwrite: bool,
}

impl DeepLakeVectorStore {
    pub fn new(
        dataset_path: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            dataset_path: dataset_path.into(),
            api_key: None,
            client: reqwest::Client::new(),
            embeddings,
            overwrite: false,
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        if let Some(ref key) = self.api_key {
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key)).unwrap(),
            );
        }
        headers
    }

    fn base_url(&self) -> String {
        "https://api.activeloop.ai/v2".into()
    }
}

#[async_trait]
impl VectorStore for DeepLakeVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("deeplake_{}", i))
            .collect();

        let mut samples = Vec::new();
        for (i, emb) in embeddings.iter().enumerate() {
            let mut meta = json!({"text": texts[i]});
            if let Some(ref metas) = metadatas {
                if let Some(m) = metas.get(i) {
                    if let Ok(v) = serde_json::to_value(m) {
                        meta = v;
                        meta["text"] = json!(texts[i]);
                    }
                }
            }
            samples.push(json!({
                "id": ids[i],
                "embedding": emb,
                "text": texts[i],
                "metadata": meta,
            }));
        }

        let body = json!({
            "dataset_path": self.dataset_path,
            "samples": samples,
            "overwrite": self.overwrite,
        });

        let url = format!("{}/datasets/ingest", self.base_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("DeepLake add error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "DeepLake add failed: {}",
                resp.status()
            )));
        }

        Ok(ids)
    }

    async fn add_documents(&self, docs: Vec<Document>) -> Result<Vec<String>> {
        let texts: Vec<String> = docs.iter().map(|d| d.page_content.clone()).collect();
        let metadatas: Vec<HashMap<String, Value>> =
            docs.iter().map(|d| d.metadata.clone()).collect();
        self.add_texts(texts, Some(metadatas)).await
    }

    async fn similarity_search(&self, query: &str, k: usize) -> Result<Vec<Document>> {
        let embedding = self.embeddings.embed_query(query).await?;
        self.similarity_search_by_vector(embedding, k).await
    }

    async fn similarity_search_by_vector(
        &self,
        embedding: Vec<f32>,
        k: usize,
    ) -> Result<Vec<Document>> {
        let body = json!({
            "dataset_path": self.dataset_path,
            "embedding": embedding,
            "k": k,
        });

        let url = format!("{}/datasets/query", self.base_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("DeepLake query error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "DeepLake query failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("DeepLake parse error: {}", e)))?;

        let matches = result["matches"].as_array().cloned().unwrap_or_default();
        let mut docs = Vec::new();

        for m in matches {
            let text = m["text"].as_str().unwrap_or("").to_string();
            let score = m["score"].as_f64().unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(score);

            if let Some(meta_obj) = m["metadata"].as_object() {
                let mut metadata = HashMap::new();
                for (k, v) in meta_obj {
                    metadata.insert(k.clone(), v.clone());
                }
                doc.metadata = metadata;
            }

            docs.push(doc);
        }

        Ok(docs)
    }

    async fn similarity_search_with_score(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        let docs = self.similarity_search(query, k).await?;
        Ok(docs
            .into_iter()
            .map(|d| {
                let score = d.score.unwrap_or(0.0);
                (d, score)
            })
            .collect())
    }

    async fn max_marginal_relevance_search(
        &self,
        query: &str,
        k: usize,
        fetch_k: usize,
        lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        let embedding = self.embeddings.embed_query(query).await?;
        let docs = self
            .similarity_search_by_vector(embedding.clone(), fetch_k)
            .await?;
        if docs.is_empty() {
            return Ok(Vec::new());
        }
        let texts: Vec<String> = docs.iter().map(|d| d.page_content.clone()).collect();
        let doc_embeddings = self.embeddings.embed_documents(&texts).await?;
        let selected = max_marginal_relevance(&embedding, &doc_embeddings, k, lambda_mult);
        Ok(selected.into_iter().map(|i| docs[i].clone()).collect())
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let body = json!({
            "dataset_path": self.dataset_path,
            "ids": ids,
        });

        let url = format!("{}/datasets/delete", self.base_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("DeepLake delete error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "DeepLake delete failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
