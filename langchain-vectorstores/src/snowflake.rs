//! Snowflake vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::cosine_similarity;

pub struct SnowflakeVectorStore {
    account: String,
    database: String,
    schema: String,
    table: String,
    user: String,
    password: String,
    warehouse: Option<String>,
    role: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl SnowflakeVectorStore {
    pub fn new(
        account: impl Into<String>,
        database: impl Into<String>,
        schema: impl Into<String>,
        table: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            account: account.into(),
            database: database.into(),
            schema: schema.into(),
            table: table.into(),
            user: user.into(),
            password: password.into(),
            warehouse: None,
            role: None,
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    pub fn with_warehouse(mut self, warehouse: impl Into<String>) -> Self {
        self.warehouse = Some(warehouse.into());
        self
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    fn token_url(&self) -> String {
        format!("https://{}.snowflakecomputing.com/oauth/token", self.account)
    }

    fn sql_url(&self) -> String {
        format!(
            "https://{}.snowflakecomputing.com/api/v2/statements",
            self.account
        )
    }

    fn full_table(&self) -> String {
        format!("{}.{}.{}", self.database, self.schema, self.table)
    }

    async fn authenticate(&self) -> Result<String> {
        let body = json!({
            "grant_type": "password",
            "username": self.user,
            "password": self.password,
        });
        let resp = self
            .client
            .post(&self.token_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Snowflake auth error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Snowflake auth failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Snowflake parse error: {}", e)))?;

        result["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ChainError::VectorStoreError("No access token".into()))
    }

    fn sql_headers(&self, token: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Accept",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }

    async fn execute_sql(&self, sql: &str) -> Result<Value> {
        let token = self.authenticate().await?;
        let mut body = json!({
            "statement": sql,
            "database": self.database,
            "schema": self.schema,
        });
        if let Some(ref wh) = self.warehouse {
            body["warehouse"] = json!(wh);
        }
        if let Some(ref role) = self.role {
            body["role"] = json!(role);
        }

        let resp = self
            .client
            .post(&self.sql_url())
            .headers(self.sql_headers(&token))
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Snowflake SQL error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Snowflake SQL failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Snowflake parse error: {}", e)))
    }
}

#[async_trait]
impl VectorStore for SnowflakeVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("snowflake_{}", i))
            .collect();

        for (i, emb) in embeddings.iter().enumerate() {
            let metadata_json = if let Some(ref metas) = metadatas {
                if let Some(m) = metas.get(i) {
                    serde_json::to_string(m).unwrap_or_else(|_| "{}".into())
                } else {
                    "{}".into()
                }
            } else {
                "{}".into()
            };

            let emb_str: Vec<String> = emb.iter().map(|v| v.to_string()).collect();
            let emb_array = format!("[{}]", emb_str.join(","));
            let text_escaped = texts[i].replace('\'', "''");
            let metadata_escaped = metadata_json.replace('\'', "''");

            let sql = format!(
                "INSERT INTO {} (id, text, embedding, metadata) VALUES ('{}', '{}', {}, '{}')",
                self.full_table(),
                ids[i],
                text_escaped,
                emb_array,
                metadata_escaped,
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
        let emb_str: Vec<String> = embedding.iter().map(|v| v.to_string()).collect();
        let emb_array = format!("[{}]", emb_str.join(","));

        let sql = format!(
            "SELECT id, text, metadata, VECTOR_COSINE_SIMILARITY(embedding, {}) AS score \
             FROM {} ORDER BY score DESC LIMIT {}",
            emb_array,
            self.full_table(),
            k
        );

        let result = self.execute_sql(&sql).await?;
        let rows = result["data"].as_array().cloned().unwrap_or_default();
        let columns = result["resultSetMetaData"]["rowType"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut docs = Vec::new();
        for row in rows {
            let values = row.as_array().cloned().unwrap_or_default();
            let mut doc = Document::new("");
            let mut score = 0.0_f32;

            for (ci, col) in columns.iter().enumerate() {
                match col["name"].as_str() {
                    Some("TEXT") => {
                        doc.page_content = values
                            .get(ci)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    }
                    Some("METADATA") => {
                        if let Some(meta_str) = values.get(ci).and_then(|v| v.as_str()) {
                            if let Ok(parsed) =
                                serde_json::from_str::<HashMap<String, Value>>(meta_str)
                            {
                                doc.metadata = parsed;
                            }
                        }
                    }
                    Some("SCORE") => {
                        score = values.get(ci).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32
                    }
                    _ => {}
                }
            }

            doc.score = Some(score);
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

        let scores: Vec<f32> = doc_embeddings
            .iter()
            .map(|e| cosine_similarity(&embedding, e))
            .collect();

        let mut selected = Vec::new();
        let mut indices: Vec<usize> = (0..docs.len()).collect();

        if let Some((idx, _)) = scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            selected.push(idx);
            indices.retain(|&i| i != idx);
        }

        while selected.len() < k.min(docs.len()) && !indices.is_empty() {
            let mut mmr_scores: Vec<(usize, f32)> = indices
                .iter()
                .map(|&i| {
                    let sim = scores[i];
                    let max_sim = selected
                        .iter()
                        .map(|&s| cosine_similarity(&doc_embeddings[i], &doc_embeddings[s]))
                        .fold(f32::NEG_INFINITY, f32::max);
                    (i, lambda_mult * sim - (1.0 - lambda_mult) * max_sim)
                })
                .collect();

            mmr_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            if let Some((next, _)) = mmr_scores.first() {
                let next = *next;
                selected.push(next);
                indices.retain(|&i| i != next);
            }
        }

        Ok(selected.into_iter().map(|i| docs[i].clone()).collect())
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let id_list = ids
            .iter()
            .map(|id| format!("'{}'", id))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "DELETE FROM {} WHERE id IN ({})",
            self.full_table(),
            id_list
        );
        self.execute_sql(&sql).await?;
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
