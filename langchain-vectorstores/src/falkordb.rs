//! FalkorDB vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct FalkorDBVectorStore {
    host: String,
    port: u16,
    graph: String,
    password: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl FalkorDBVectorStore {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        graph: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            graph: graph.into(),
            password: None,
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    fn post_url(&self) -> String {
        format!("http://{}:{}/graph/{}", self.host, self.port, self.graph)
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref pass) = self.password {
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", pass)).unwrap(),
            );
        }
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }

    async fn query(&self, query: &str) -> Result<Value> {
        let body = json!({ "query": query });
        let resp = self
            .client
            .post(&self.post_url())
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("FalkorDB request error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "FalkorDB request failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("FalkorDB parse error: {}", e)))
    }

    fn emb_to_array(emb: &[f32]) -> String {
        let inner: Vec<String> = emb.iter().map(|v| v.to_string()).collect();
        format!("[{}]", inner.join(","))
    }
}

#[async_trait]
impl VectorStore for FalkorDBVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("falkordb_{}", i))
            .collect();

        for (i, emb) in embeddings.iter().enumerate() {
            let meta_json = if let Some(ref metas) = metadatas {
                if let Some(m) = metas.get(i) {
                    serde_json::to_string(m).unwrap_or_else(|_| "{}".into())
                } else {
                    "{}".into()
                }
            } else {
                "{}".into()
            };

            let query = format!(
                "CREATE (:Chunk {{ id: '{}', text: {}, embedding: {}, metadata: {} }})",
                ids[i],
                quote_string(&texts[i]),
                Self::emb_to_array(emb),
                quote_string(&meta_json),
            );
            self.query(&query).await?;
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
        let emb_array = Self::emb_to_array(&embedding);
        let query = format!(
            "MATCH (c:Chunk) \
             WITH c, vector.cos similarity(c.embedding, {}) AS score \
             ORDER BY score DESC \
             LIMIT {} \
             RETURN c.id AS id, c.text AS text, c.metadata AS metadata, score",
            emb_array, k,
        );

        let result = self.query(&query).await?;
        let data = result["data"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut docs = Vec::new();

        for row in data {
            let values = row.as_array().cloned().unwrap_or_default();
            if values.len() < 4 {
                continue;
            }
            let text = values[1].as_str().unwrap_or("").to_string();
            let score = values[3].as_f64().unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(score);

            if let Some(meta_str) = values[2].as_str() {
                if let Ok(parsed) = serde_json::from_str::<HashMap<String, Value>>(meta_str) {
                    doc.metadata = parsed;
                }
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
        let id_list: Vec<String> = ids
            .iter()
            .map(|id| format!("'{}'", id))
            .collect();
        let query = format!(
            "MATCH (c:Chunk) WHERE c.id IN [{}] DELETE c",
            id_list.join(",")
        );
        self.query(&query).await?;
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}

fn quote_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}
