//! Pinecone vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;

pub struct PineconeVectorStore {
    api_key: String,
    environment: String,
    index_name: String,
    project_id: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl PineconeVectorStore {
    pub fn new(
        api_key: impl Into<String>,
        environment: impl Into<String>,
        index_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            environment: environment.into(),
            index_name: index_name.into(),
            project_id: None,
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    fn base_url(&self) -> String {
        format!(
            "https://{}-{}.svc.{}.pinecone.io",
            self.index_name,
            self.project_id.as_deref().unwrap_or("unknown"),
            self.environment
        )
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Api-Key",
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers
    }
}

#[async_trait]
impl VectorStore for PineconeVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;

        let ids: Vec<String> = (0..texts.len()).map(|i| format!("pinecone_{}", i)).collect();

        let mut vectors = Vec::new();
        for (i, emb) in embeddings.iter().enumerate() {
            let mut metadata = json!({"text": texts[i]});
            if let Some(ref metas) = metadatas {
                if let Some(meta) = metas.get(i) {
                    metadata = serde_json::to_value(meta).unwrap_or(metadata);
                    metadata["text"] = json!(texts[i]);
                }
            }
            vectors.push(json!({
                "id": ids[i],
                "values": emb,
                "metadata": metadata,
            }));
        }

        let body = json!({ "vectors": vectors });

        let url = format!("{}/vectors/upsert", self.base_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Pinecone upsert error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Pinecone upsert failed: {}",
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
            "vector": embedding,
            "topK": k,
            "includeMetadata": true,
            "includeValues": false,
        });

        let url = format!("{}/query", self.base_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Pinecone query error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Pinecone query failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Pinecone parse error: {}", e)))?;

        let matches = result["matches"].as_array().cloned().unwrap_or_default();
        let mut docs = Vec::new();

        for m in matches {
            let text = m["metadata"]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let score = m["score"].as_f64().unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(score);

            if let Some(meta_obj) = m["metadata"].as_object() {
                let mut metadata = HashMap::new();
                for (k, v) in meta_obj {
                    if k != "text" {
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
            "MMR search not supported for Pinecone".into(),
        ))
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let body = json!({ "ids": ids });
        let url = format!("{}/vectors/delete", self.base_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Pinecone delete error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Pinecone delete failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
