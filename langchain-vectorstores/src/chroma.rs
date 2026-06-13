//! ChromaDB vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct ChromaVectorStore {
    url: String,
    collection_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
    collection_id: Option<String>,
}

impl ChromaVectorStore {
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
            collection_id: None,
        }
    }

    async fn ensure_collection(&self) -> Result<String> {
        if let Some(ref id) = self.collection_id {
            return Ok(id.clone());
        }

        let url = format!("{}/api/v1/collections", self.url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Chroma request error: {}", e)))?;

        if resp.status().is_success() {
            let collections: Vec<Value> = resp
                .json()
                .await
                .map_err(|e| ChainError::VectorStoreError(format!("Chroma parse error: {}", e)))?;
            for col in collections {
                if col["name"].as_str() == Some(&self.collection_name) {
                    let id = col["id"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    return Ok(id);
                }
            }
        }

        let create_url = format!("{}/api/v1/collections", self.url);
        let create_resp = self
            .client
            .post(&create_url)
            .json(&json!({"name": self.collection_name}))
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Chroma create error: {}", e)))?;

        if create_resp.status().is_success() {
            let col: Value = create_resp
                .json()
                .await
                .map_err(|e| ChainError::VectorStoreError(format!("Chroma parse error: {}", e)))?;
            Ok(col["id"]
                .as_str()
                .unwrap_or("")
                .to_string())
        } else {
            Err(ChainError::VectorStoreError(format!(
                "Chroma create collection failed: {}",
                create_resp.status()
            )))
        }
    }
}

#[async_trait]
impl VectorStore for ChromaVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let ids: Vec<String> = (0..texts.len()).map(|i| format!("chroma_{}", i)).collect();
        let metadatas_vec: Vec<Value> = match metadatas {
            Some(ref metas) => metas
                .iter()
                .map(|m| serde_json::to_value(m).unwrap_or(json!({})))
                .collect(),
            None => vec![json!({}); texts.len()],
        };

        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let collection_id = self.ensure_collection().await?;

        let body = json!({
            "ids": ids,
            "embeddings": embeddings,
            "metadatas": metadatas_vec,
            "documents": texts,
        });

        let url = format!("{}/api/v1/collections/{}/add", self.url, collection_id);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Chroma add error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Chroma add failed: {}",
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
        let collection_id = self.ensure_collection().await?;

        let body = json!({
            "query_embeddings": [embedding],
            "n_results": k,
            "include": ["documents", "metadatas", "distances"]
        });

        let url = format!("{}/api/v1/collections/{}/query", self.url, collection_id);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Chroma query error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Chroma query failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Chroma parse error: {}", e)))?;

        let documents = result["documents"][0]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let metadatas = result["metadatas"][0]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let distances = result["distances"][0]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut docs = Vec::new();
        for (i, doc_text) in documents.iter().enumerate() {
            let mut doc = Document::new(doc_text.as_str().unwrap_or(""));
            if let Some(meta) = metadatas.get(i) {
                if let Some(obj) = meta.as_object() {
                    doc.metadata = obj.clone().into_iter().collect();
                }
            }
            if let Some(dist) = distances.get(i).and_then(|v| v.as_f64()) {
                doc = doc.with_score(dist as f32);
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
        let collection_id = self.ensure_collection().await?;
        let body = json!({ "ids": ids });
        let url = format!("{}/api/v1/collections/{}/delete", self.url, collection_id);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Chroma delete error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Chroma delete failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
