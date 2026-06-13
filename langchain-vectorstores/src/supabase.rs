//! Supabase vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;

pub struct SupabaseVectorStore {
    url: String,
    api_key: String,
    table_name: String,
    query_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl SupabaseVectorStore {
    pub fn new(
        url: impl Into<String>,
        api_key: impl Into<String>,
        table_name: impl Into<String>,
        query_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            api_key: api_key.into(),
            table_name: table_name.into(),
            query_name: query_name.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "apikey",
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }
}

#[async_trait]
impl VectorStore for SupabaseVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len()).map(|i| format!("supa_{}", i)).collect();

        let mut records = Vec::new();
        for (i, emb) in embeddings.iter().enumerate() {
            let mut record = json!({
                "id": ids[i],
                "content": texts[i],
                "embedding": emb,
            });
            if let Some(ref metas) = metadatas {
                if let Some(meta) = metas.get(i) {
                    if let Some(obj) = record.as_object_mut() {
                        for (k, v) in meta {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            records.push(record);
        }

        let body = json!({ "records": records });
        let url = format!("{}/rest/v1/{}", self.url, self.table_name);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Supabase insert error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Supabase insert failed: {}",
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
            "query_embedding": embedding,
            "match_count": k,
        });

        let url = format!(
            "{}/rest/v1/rpc/{}",
            self.url, self.query_name
        );
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Supabase search error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Supabase search failed: {}",
                resp.status()
            )));
        }

        let results: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Supabase parse error: {}", e)))?;

        let items = results.as_array().cloned().unwrap_or_default();
        let mut docs = Vec::new();

        for item in items {
            let text = item["content"].as_str().unwrap_or("").to_string();
            let similarity = item["similarity"].as_f64().unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(similarity);

            if let Some(obj) = item.as_object() {
                let mut metadata = HashMap::new();
                for (k, v) in obj {
                    if k != "content" && k != "embedding" && k != "similarity" {
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
            "MMR search not supported for Supabase".into(),
        ))
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Err(ChainError::VectorStoreError(
            "Supabase delete not implemented".into(),
        ))
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
