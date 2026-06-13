//! Aerospike vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct AerospikeVectorStore {
    host: String,
    port: u16,
    namespace: String,
    set_name: String,
    index_name: Option<String>,
    user: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl AerospikeVectorStore {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        namespace: impl Into<String>,
        set_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            namespace: namespace.into(),
            set_name: set_name.into(),
            index_name: None,
            user: None,
            password: None,
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    pub fn with_credentials(
        mut self,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.user = Some(user.into());
        self.password = Some(password.into());
        self
    }

    pub fn with_index_name(mut self, index_name: impl Into<String>) -> Self {
        self.index_name = Some(index_name.into());
        self
    }

    fn rest_url(&self) -> String {
        format!("http://{}:{}/v1/vector", self.host, self.port)
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let (Some(ref user), Some(ref pass)) = (&self.user, &self.password) {
            let auth = format!("{}:{}", user, pass);
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!(
                    "Basic {}",
                    base64_encode(&auth)
                ))
                .unwrap(),
            );
        }
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }

    fn index(&self) -> &str {
        self.index_name.as_deref().unwrap_or("vector_index")
    }
}

fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[async_trait]
impl VectorStore for AerospikeVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("aerospike_{}", i))
            .collect();

        let mut records = Vec::new();
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
            records.push(json!({
                "key": ids[i],
                "namespace": self.namespace,
                "set": self.set_name,
                "vector": emb,
                "metadata": meta,
            }));
        }

        let body = json!({ "records": records });

        let url = format!("{}/records", self.rest_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Aerospike add error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Aerospike add failed: {}",
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
            "namespace": self.namespace,
            "index": self.index(),
            "vector": embedding,
            "k": k,
            "includeMetadata": true,
        });

        let url = format!("{}/search", self.rest_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Aerospike search error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Aerospike search failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Aerospike parse error: {}", e)))?;

        let matches = result["results"].as_array().cloned().unwrap_or_default();
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
            "namespace": self.namespace,
            "set": self.set_name,
            "keys": ids,
        });

        let url = format!("{}/records/delete", self.rest_url());
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Aerospike delete error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Aerospike delete failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
