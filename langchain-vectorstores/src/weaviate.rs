//! Weaviate vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::{json, Value};

use crate::traits::VectorStore;

pub struct WeaviateVectorStore {
    url: String,
    class_name: String,
    client: reqwest::Client,
    api_key: Option<String>,
    embeddings: Arc<dyn Embeddings>,
}

impl WeaviateVectorStore {
    pub fn new(
        url: impl Into<String>,
        class_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            class_name: class_name.into(),
            client: reqwest::Client::new(),
            api_key: None,
            embeddings,
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        if let Some(ref key) = self.api_key {
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key)).unwrap(),
            );
        }
        headers
    }
}

#[async_trait]
impl VectorStore for WeaviateVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;
        let ids: Vec<String> = (0..texts.len()).map(|i| format!("weaviate_{}", i)).collect();

        let mut objects = Vec::new();
        for (i, emb) in embeddings.iter().enumerate() {
            let mut properties = json!({
                "text": texts[i],
                "doc_id": ids[i],
            });
            if let Some(ref metas) = metadatas {
                if let Some(meta) = metas.get(i) {
                    if let Some(obj) = properties.as_object_mut() {
                        for (k, v) in meta {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }

            objects.push(json!({
                "class": self.class_name,
                "id": ids[i],
                "vector": emb,
                "properties": properties,
            }));
        }

        let body = json!({ "objects": objects });
        let url = format!("{}/v1/objects", self.url);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Weaviate insert error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Weaviate insert failed: {}",
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
        let near_vec_str = format!("{{vector:{:?}}}", embedding);
        let gql = format!(
            "{{ Get {{{}(nearVector:{},limit:{}){{text _additional{{distance}}}}}} }}",
            self.class_name, near_vec_str, k
        );
        let graphql_query = json!({ "query": gql });

        let url = format!("{}/v1/graphql", self.url);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&graphql_query)
            .send()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Weaviate search error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::VectorStoreError(format!(
                "Weaviate search failed: {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Weaviate parse error: {}", e)))?;

        let items = result["data"]["Get"][&self.class_name]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut docs = Vec::new();

        for item in items {
            let text = item["text"].as_str().unwrap_or("").to_string();
            let distance = item["_additional"]["distance"]
                .as_f64()
                .unwrap_or(0.0) as f32;
            let mut doc = Document::new(text).with_score(1.0 - distance);

            if let Some(obj) = item.as_object() {
                let mut metadata = HashMap::new();
                for (k, v) in obj {
                    if k != "text" && k != "_additional" {
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
            "MMR search not supported for Weaviate".into(),
        ))
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Err(ChainError::VectorStoreError(
            "Weaviate delete not implemented via REST API".into(),
        ))
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
