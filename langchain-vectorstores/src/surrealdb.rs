//! SurrealDB vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct SurrealDBVectorStore {
    url: String,
    namespace: String,
    database: String,
    table: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl SurrealDBVectorStore {
    pub fn new(
        url: impl Into<String>,
        namespace: impl Into<String>,
        database: impl Into<String>,
        table: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            namespace: namespace.into(),
            database: database.into(),
            table: table.into(),
            username: None,
            password: None,
            token: None,
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "NS",
            reqwest::header::HeaderValue::from_str(&self.namespace).unwrap(),
        );
        headers.insert(
            "DB",
            reqwest::header::HeaderValue::from_str(&self.database).unwrap(),
        );
        if let Some(ref token) = self.token {
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
        } else if let (Some(ref user), Some(ref pass)) = (&self.username, &self.password) {
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
        headers
    }

    fn sql_url(&self) -> String {
        format!("{}/sql", self.url.trim_end_matches('/'))
    }

    async fn execute_sql(&self, sql: &str) -> Result<Value> {
        let resp = self
            .client
            .post(&self.sql_url())
            .headers(self.headers())
            .body(sql.to_string())
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("SurrealDB request error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "SurrealDB request failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("SurrealDB parse error: {}", e)))
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

fn emb_to_array(emb: &[f32]) -> String {
    let inner: Vec<String> = emb.iter().map(|v| v.to_string()).collect();
    format!("[{}]", inner.join(","))
}

#[async_trait]
impl VectorStore for SurrealDBVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("surrealdb_{}", i))
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

            let sql = format!(
                "CREATE {}:{} CONTENT {{ id: '{}', text: {}, embedding: {}, metadata: {} }}",
                self.table,
                ids[i],
                ids[i],
                quote_string(&texts[i]),
                emb_to_array(emb),
                meta_json,
            );
            self.execute_sql(&sql).await?;
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
        let emb_array = emb_to_array(&embedding);
        let sql = format!(
            "SELECT id, text, metadata, vector::distance::cosine(embedding, {}) AS score \
             FROM {} ORDER BY score ASC LIMIT {}",
            emb_array,
            self.table,
            k,
        );

        let result = self.execute_sql(&sql).await?;
        let rows = result.as_array().cloned().unwrap_or_default();
        let mut docs = Vec::new();

        for entry in rows {
            let result_obj = entry["result"].as_object().cloned().unwrap_or_default();
            let text = result_obj
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let score = result_obj
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(1.0 - score);

            if let Some(meta_val) = result_obj.get("metadata") {
                if let Some(meta_obj) = meta_val.as_object() {
                    let mut metadata = HashMap::new();
                    for (k, v) in meta_obj {
                        metadata.insert(k.clone(), v.clone());
                    }
                    doc.metadata = metadata;
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
            .map(|id| format!("{}:{}", self.table, id))
            .collect();
        let sql = format!("DELETE FROM {} WHERE id IN [{}]", self.table, id_list.join(","));
        self.execute_sql(&sql).await?;
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}

fn quote_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}
