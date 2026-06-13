//! Qdrant vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    VectorParamsBuilder,
};
use serde_json::Value;

use crate::traits::VectorStore;

pub struct QdrantVectorStore {
    client: Qdrant,
    collection_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl QdrantVectorStore {
    pub fn new(
        client: Qdrant,
        collection_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            client,
            collection_name: collection_name.into(),
            embeddings,
        }
    }

    pub async fn from_url(
        url: impl Into<String>,
        collection_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Result<Self> {
        let client = Qdrant::from_url(&url.into())
            .build()
            .map_err(|e| ChainError::VectorStoreError(format!("Qdrant client error: {}", e)))?;
        Ok(Self {
            client,
            collection_name: collection_name.into(),
            embeddings,
        })
    }

    async fn ensure_collection(&self, dim: usize) -> Result<()> {
        let collections = self
            .client
            .list_collections()
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Qdrant list error: {}", e)))?;

        let exists = collections.collections.iter().any(|c| c.name == self.collection_name);

        if !exists {
            let create = CreateCollectionBuilder::new(self.collection_name.clone())
                .vectors_config(VectorParamsBuilder::new(dim as u64, Distance::Cosine));
            self.client
                .create_collection(create)
                .await
                .map_err(|e| ChainError::VectorStoreError(format!("Qdrant create error: {}", e)))?;
        }
        Ok(())
    }
}

#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let embeddings = self.embeddings.embed_documents(&texts).await?;

        if !embeddings.is_empty() {
            self.ensure_collection(embeddings[0].len()).await?;
        }

        let ids: Vec<String> = (0..texts.len()).map(|i| format!("qdrant_{}", i)).collect();
        let mut points = Vec::new();

        for (i, emb) in embeddings.iter().enumerate() {
            let mut payload: HashMap<String, Value> = HashMap::new();
            payload.insert("text".to_string(), Value::String(texts[i].clone()));
            if let Some(ref metas) = metadatas {
                if let Some(meta) = metas.get(i) {
                    for (k, v) in meta {
                        payload.insert(k.clone(), v.clone());
                    }
                }
            }

            let point = PointStruct::new(ids[i].clone(), emb.clone(), payload);
            points.push(point);
        }

        self.client
            .upsert_points(UpsertPointsBuilder::new(self.collection_name.clone(), points))
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Qdrant upsert error: {}", e)))?;

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
        let search = SearchPointsBuilder::new(self.collection_name.clone(), embedding, k as u64)
            .with_payload(true);

        let result = self
            .client
            .search_points(search)
            .await
            .map_err(|e| ChainError::VectorStoreError(format!("Qdrant search error: {}", e)))?;

        let mut docs = Vec::new();
        for point in result.result {
            let text = point.payload.get("text")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let score = point.score;
            let mut doc = Document::new(text).with_score(score);

            let mut metadata = HashMap::new();
            for (k, v) in &point.payload {
                if k != "text" {
                    metadata.insert(k.clone(), serde_json::to_value(v).unwrap_or(Value::Null));
                }
            }
            doc.metadata = metadata;
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
            "MMR search not supported for Qdrant".into(),
        ))
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        #[allow(unused_imports)]
        use qdrant_client::qdrant::{DeletePointsBuilder, PointsIdsList};

        #[allow(unused_variables)]
        let points_ids: Vec<qdrant_client::qdrant::PointId> = ids
            .iter()
            .map(|_id| qdrant_client::qdrant::PointId {
                point_id_options: Some(
                    qdrant_client::qdrant::point_id::PointIdOptions::Num(0),
                ),
            })
            .collect();

        let _ = points_ids; // Placeholder for actual deletion
        Err(ChainError::VectorStoreError(
            "Qdrant delete requires point ID handling - use client directly".into(),
        ))
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
