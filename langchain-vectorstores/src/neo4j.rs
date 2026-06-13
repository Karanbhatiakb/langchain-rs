//! Neo4j vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;

pub struct Neo4jVectorStore {
    url: String,
    database: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl Neo4jVectorStore {
    pub fn new(
        url: impl Into<String>,
        database: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            database: database.into(),
            username: None,
            password: None,
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    pub fn with_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }
}

#[async_trait]
impl VectorStore for Neo4jVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len()).map(|i| format!("neo4j_{}", i)).collect();

        for (i, emb) in embeddings.iter().enumerate() {
            let mut props = json!({
                "id": ids[i],
                "text": texts[i],
                "embedding": emb,
            });
            if let Some(ref metas) = metadatas {
                if let Some(meta) = metas.get(i) {
                    if let Some(obj) = props.as_object_mut() {
                        for (k, v) in meta {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }

            let cypher = format!(
                "CREATE (n:Document {{ id: $id, text: $text, embedding: $embedding }}) RETURN n.id"
            );
            let body = json!({
                "statements": [{
                    "statement": cypher,
                    "parameters": {
                        "id": ids[i],
                        "text": texts[i],
                        "embedding": emb,
                    }
                }]
            });

            let resp = self
                .client
                .post(&format!("{}/db/{}/tx/commit", self.url, self.database))
                .json(&body)
                .send()
                .await
                .map_err(|e| ChainError::VectorStoreError(format!("Neo4j insert error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::VectorStoreError(format!(
                    "Neo4j insert failed: {}",
                    resp.status()
                )));
            }
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
        let emb_json = serde_json::to_string(&embedding).unwrap_or_default();
        let cypher = format!(
            "MATCH (n:Document) WITH n, gds.similarity.cosine(n.embedding, {}) AS score ORDER BY score DESC LIMIT {} RETURN n.text AS text, n.id AS id, score",
            emb_json, k
        );

        let body = json!({
            "statements": [{
                "statement": cypher
            }]
        });

        let resp = self
            .client
            .post(&format!("{}/db/{}/tx/commit", self.url, self.database))
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Neo4j search error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Neo4j search failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Neo4j parse error: {}", e)))?;

        let mut docs = Vec::new();
        if let Some(rows) = result["results"][0]["data"].as_array() {
            for row in rows {
                if let Some(row_data) = row["row"].as_array() {
                    if row_data.len() >= 3 {
                        let text = row_data[0].as_str().unwrap_or("").to_string();
                        let score = row_data[2].as_f64().unwrap_or(0.0) as f32;
                        docs.push(Document::new(text).with_score(score));
                    }
                }
            }
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
            "MMR search not supported for Neo4j".into(),
        ))
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Err(ChainError::VectorStoreError(
            "Neo4j delete not implemented".into(),
        ))
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
