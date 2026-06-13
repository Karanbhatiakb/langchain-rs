//! MySQL vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct MySQLVectorStore {
    host: String,
    port: u16,
    database: String,
    table: String,
    user: String,
    password: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl MySQLVectorStore {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
        table: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            database: database.into(),
            table: table.into(),
            user: user.into(),
            password: password.into(),
            client: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(false)
                .build()
                .unwrap(),
            embeddings,
        }
    }

    fn rest_url(&self) -> String {
        format!(
            "https://{}:{}/api/query/{}",
            self.host, self.port, self.database
        )
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth = format!("{}:{}", self.user, self.password);
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!(
                "Basic {}",
                base64_encode(&auth)
            ))
            .unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }

    async fn execute(&self, sql: &str) -> Result<Value> {
        let body = json!({ "sql": sql });
        let resp = self
            .client
            .post(&self.rest_url())
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("MySQL request error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "MySQL request failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("MySQL parse error: {}", e)))
    }

    fn emb_to_array(emb: &[f32]) -> String {
        let inner: Vec<String> = emb.iter().map(|v| v.to_string()).collect();
        format!("[{}]", inner.join(","))
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
impl VectorStore for MySQLVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("mysql_{}", i))
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
                "INSERT INTO {} (id, text, embedding, metadata) VALUES ('{}', {}, {}, '{}')",
                self.table,
                ids[i],
                quote_string(&texts[i]),
                Self::emb_to_array(emb),
                meta_json.replace('\'', "''"),
            );
            self.execute(&sql).await?;
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
        let sql = format!(
            "SELECT id, text, metadata, \
             (1 - COALESCE( \
               (SELECT SQRT(GREATEST(0, SUM(POW(e1.val - e2.val, 2)))) / \
                       GREATEST(SQRT(SUM(e1.val * e1.val)) * SQRT(SUM(e2.val * e2.val)), 1e-10) \
                FROM JSON_TABLE({}, '$[*]' COLUMNS (val DOUBLE PATH '$')) AS e1, \
                JSON_TABLE(embedding, '$[*]' COLUMNS (val DOUBLE PATH '$')) AS e2), 1) \
             ) AS score \
             FROM {} \
             ORDER BY score DESC LIMIT {}",
            emb_array, self.table, k,
        );

        let result = self.execute(&sql).await?;
        let rows = result["rows"].as_array().cloned().unwrap_or_default();
        let mut docs = Vec::new();

        for row in rows {
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
        let id_list: Vec<String> = ids.iter().map(|id| format!("'{}'", id)).collect();
        let sql = format!(
            "DELETE FROM {} WHERE id IN ({})",
            self.table,
            id_list.join(",")
        );
        self.execute(&sql).await?;
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}

fn quote_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}
