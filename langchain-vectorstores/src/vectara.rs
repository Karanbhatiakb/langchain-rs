//! Vectara vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct VectaraVectorStore {
    customer_id: String,
    corpus_id: String,
    api_key: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl VectaraVectorStore {
    pub fn new(
        customer_id: impl Into<String>,
        corpus_id: impl Into<String>,
        api_key: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            customer_id: customer_id.into(),
            corpus_id: corpus_id.into(),
            api_key: api_key.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "customer-id",
            reqwest::header::HeaderValue::from_str(&self.customer_id).unwrap(),
        );
        headers
    }
}

#[async_trait]
impl VectorStore for VectaraVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("vectara_{}", i))
            .collect();

        let mut documents = Vec::new();
        for (i, text) in texts.iter().enumerate() {
            let mut metadata = json!({});
            if let Some(ref metas) = metadatas {
                if let Some(m) = metas.get(i) {
                    if let Ok(v) = serde_json::to_value(m) {
                        metadata = v;
                    }
                }
            }
            documents.push(json!({
                "id": ids[i],
                "text": text,
                "metadata": metadata,
            }));
        }

        let body = json!({
            "documents": documents,
        });

        let url = format!(
            "https://api.vectara.io/v1/corpi/{}/documents",
            self.corpus_id
        );
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Vectara add error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Vectara add failed: {}",
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
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        Err(ChainError::VectorStoreError(
            "Vectara does not support vector-based search; use query-based search via similarity_search".into(),
        ))
    }

    async fn similarity_search_with_score(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        let body = json!({
            "query": [{
                "query": query,
                "numResults": k,
                "corpusKey": [{
                    "corpusId": self.corpus_id
                }]
            }]
        });

        let url = "https://api.vectara.io/v1/query";
        let resp = self
            .client
            .post(url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Vectara query error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Vectara query failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Vectara parse error: {}", e)))?;

        let response_set = result["responseSet"]
            .as_array()
            .and_then(|rs| rs.first())
            .cloned()
            .unwrap_or_default();

        let documents = response_set["documents"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let scores = response_set["scores"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut docs = Vec::new();
        for (i, doc_val) in documents.iter().enumerate() {
            let text = doc_val["text"].as_str().unwrap_or("").to_string();
            let score = scores
                .get(i)
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(score);

            if let Some(meta_obj) = doc_val["metadata"].as_object() {
                let mut metadata = HashMap::new();
                for (k, v) in meta_obj {
                    metadata.insert(k.clone(), v.clone());
                }
                doc.metadata = metadata;
            }
            docs.push((doc, score));
        }

        Ok(docs)
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
            .similarity_search_with_score(query, fetch_k)
            .await?;
        if docs.is_empty() {
            return Ok(Vec::new());
        }
        let texts: Vec<String> = docs.iter().map(|(d, _)| d.page_content.clone()).collect();
        let doc_embeddings = self.embeddings.embed_documents(&texts).await?;
        let selected = max_marginal_relevance(&embedding, &doc_embeddings, k, lambda_mult);
        Ok(selected
            .into_iter()
            .map(|i| docs[i].0.clone())
            .collect())
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let url = format!(
            "https://api.vectara.io/v1/corpi/{}/documents",
            self.corpus_id
        );
        let resp = self
            .client
            .delete(&url)
            .headers(self.headers())
            .json(&json!({ "ids": ids }))
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Vectara delete error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Vectara delete failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
