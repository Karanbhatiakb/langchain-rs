//! SingleStore vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;

pub struct SingleStoreVectorStore {
    url: String,
    table_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl SingleStoreVectorStore {
    pub fn new(
        url: impl Into<String>,
        table_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            table_name: table_name.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for SingleStoreVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len()).map(|i| format!("ss_{}", i)).collect();

        let mut rows = Vec::new();
        for (i, emb) in embeddings.iter().enumerate() {
            let mut row = json!({
                "id": ids[i],
                "text": texts[i],
                "vector": emb,
            });
            if let Some(ref metas) = metadatas {
                if let Some(meta) = metas.get(i) {
                    if let Some(obj) = row.as_object_mut() {
                        for (k, v) in meta {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            rows.push(row);
        }

        let body = json!({ "table": self.table_name, "rows": rows });
        let resp = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("SingleStore insert error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "SingleStore insert failed: {}",
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
            "table": self.table_name,
            "vector": embedding,
            "k": k,
        });

        let url = format!("{}/search", self.url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("SingleStore search error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "SingleStore search failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("SingleStore parse error: {}", e)))?;

        let items = result["results"].as_array().cloned().unwrap_or_default();
        let mut docs = Vec::new();

        for item in items {
            let text = item["text"].as_str().unwrap_or("").to_string();
            let score = item["score"].as_f64().unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(score);

            if let Some(obj) = item.as_object() {
                let mut metadata = HashMap::new();
                for (k, v) in obj {
                    if k != "text" && k != "vector" && k != "score" {
                        metadata.insert(k.clone(), v.clone());
                    }
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
        Ok(docs.into_iter().map(|d| {
            let score = d.score.unwrap_or(0.0);
            (d, score)
        }).collect())
    }

    async fn max_marginal_relevance_search(
        &self,
        _query: &str,
        _k: usize,
        _fetch_k: usize,
        _lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        Err(ChainError::VectorStoreError(
            "MMR search not supported for SingleStore".into(),
        ))
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Err(ChainError::VectorStoreError(
            "SingleStore delete not implemented".into(),
        ))
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
