//! Astra DB vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;
use crate::utils::max_marginal_relevance;

pub struct AstraDBVectorStore {
    database_id: String,
    region: String,
    keyspace: String,
    collection_name: String,
    token: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl AstraDBVectorStore {
    pub fn new(
        database_id: impl Into<String>,
        region: impl Into<String>,
        keyspace: impl Into<String>,
        collection_name: impl Into<String>,
        token: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            database_id: database_id.into(),
            region: region.into(),
            keyspace: keyspace.into(),
            collection_name: collection_name.into(),
            token: token.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    fn base_url(&self) -> String {
        format!(
            "https://{}-{}.apps.astra.datastax.com/api/json/v1/{}",
            self.database_id, self.region, self.keyspace
        )
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-Cassandra-Token",
            reqwest::header::HeaderValue::from_str(&self.token).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }

    #[allow(dead_code)]
    async fn ensure_collection(&self) -> Result<()> {
        let url = format!("{}/{}", self.base_url(), self.collection_name);
        let body = json!({
            "createCollection": {
                "name": self.collection_name,
                "options": {
                    "vector": {
                        "size": 1536,
                        "function": "cosine"
                    }
                }
            }
        });
        let _resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("AstraDB request error: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl VectorStore for AstraDBVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len())
            .map(|i| format!("astra_{}", i))
            .collect();

        let url = format!("{}/{}/bulk", self.base_url(), self.collection_name);
        let mut documents = Vec::new();

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
            documents.push(json!({
                "_id": ids[i],
                "$vector": emb,
                "metadata": meta,
            }));
        }

        let body = json!({ "insertDocuments": documents });
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("AstraDB add error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "AstraDB add failed: {}",
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
        let url = format!("{}/{}/vector", self.base_url(), self.collection_name);
        let body = json!({
            "find": {},
            "sort": { "$vector": embedding },
            "limit": k,
            "includeSimilarity": true,
        });

        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("AstraDB query error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "AstraDB query failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("AstraDB parse error: {}", e)))?;

        let data = result["data"]["documents"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut docs = Vec::new();
        for item in data {
            let text = item["metadata"]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let score = item["$similarity"].as_f64().unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(score);
            if let Some(meta_obj) = item["metadata"].as_object() {
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
        let url = format!("{}/{}", self.base_url(), self.collection_name);
        let body = json!({ "deleteMany": { "_id": { "$in": ids } } });
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("AstraDB delete error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "AstraDB delete failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
