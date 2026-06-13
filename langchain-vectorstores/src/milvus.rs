//! Milvus vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct MilvusVectorStore {
    url: String,
    collection_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl MilvusVectorStore {
    pub fn new(
        url: impl Into<String>,
        collection_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            collection_name: collection_name.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for MilvusVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len()).map(|i| format!("milvus_{}", i)).collect();

        let body = json!({
            "collection_name": self.collection_name,
            "vectors": embeddings,
            "texts": texts,
            "ids": ids,
            "metadatas": metadatas,
        });

        let url = format!("{}/api/v1/insert", self.url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Milvus insert error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Milvus insert failed: {}",
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
            "collection_name": self.collection_name,
            "vector": embedding,
            "topk": k,
        });

        let url = format!("{}/api/v1/search", self.url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Milvus search error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Milvus search failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Milvus parse error: {}", e)))?;

        let results = result["results"].as_array().cloned().unwrap_or_default();
        let mut docs = Vec::new();

        for item in results {
            let text = item["text"].as_str().unwrap_or("").to_string();
            let score = item["score"].as_f64().unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(score);

            if let Some(meta_obj) = item["metadata"].as_object() {
                doc.metadata = meta_obj.clone().into_iter().collect();
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
        query: &str,
        k: usize,
        fetch_k: usize,
        lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        let embedding = self.embeddings.embed_query(query).await?;
        let docs = self.similarity_search_by_vector(embedding.clone(), fetch_k).await?;
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
            "collection_name": self.collection_name,
            "ids": ids,
        });
        let url = format!("{}/api/v1/delete", self.url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Milvus delete error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Milvus delete failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
