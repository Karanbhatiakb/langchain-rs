//! SKLearn vector store implementation.
//!
//! An in-process vector store that mirrors the Python scikit-learn
//! `NearestNeighbors`-based vector store pattern.  Useful for small-scale
//! local experimentation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use parking_lot::RwLock;
use serde_json::Value;

use crate::traits::VectorStore;
use crate::utils::{cosine_similarity, max_marginal_relevance, top_k_by_score};

/// In-process vector store modeled after scikit-learn's `NearestNeighbors`.
///
/// Stores documents and vectors in memory and performs similarity search
/// via cosine similarity.
#[derive(Clone)]
pub struct SKLearnVectorStore {
    documents: Arc<RwLock<Vec<Document>>>,
    vectors: Arc<RwLock<Vec<Vec<f32>>>>,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for SKLearnVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SKLearnVectorStore")
            .field("documents", &self.documents)
            .field("vectors", &self.vectors)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl SKLearnVectorStore {
    /// Create a new `SKLearnVectorStore`.
    ///
    /// * `embeddings` — the embedding model.
    pub fn new(embeddings: Arc<dyn Embeddings>) -> Self {
        Self {
            documents: Arc::new(RwLock::new(Vec::new())),
            vectors: Arc::new(RwLock::new(Vec::new())),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for SKLearnVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let docs: Vec<Document> = texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let mut doc = Document::new(text.clone());
                if let Some(ref metas) = metadatas {
                    if let Some(meta) = metas.get(i) {
                        doc.metadata = meta.clone();
                    }
                }
                doc
            })
            .collect();

        let embeddings = self.embeddings.embed_documents(&texts).await?;

        let mut docs_guard = self.documents.write();
        let mut vecs_guard = self.vectors.write();

        let ids: Vec<String> = docs
            .iter()
            .enumerate()
            .map(|(i, _)| format!("sklearn_{}", docs_guard.len() + i))
            .collect();

        docs_guard.extend(docs);
        vecs_guard.extend(embeddings);

        Ok(ids)
    }

    async fn add_documents(&self, docs: Vec<Document>) -> Result<Vec<String>> {
        let texts: Vec<String> = docs.iter().map(|d| d.page_content.clone()).collect();
        let embeddings = self.embeddings.embed_documents(&texts).await?;

        let mut docs_guard = self.documents.write();
        let mut vecs_guard = self.vectors.write();

        let ids: Vec<String> = docs
            .iter()
            .enumerate()
            .map(|(i, _)| format!("sklearn_{}", docs_guard.len() + i))
            .collect();

        docs_guard.extend(docs);
        vecs_guard.extend(embeddings);

        Ok(ids)
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
        let vecs = self.vectors.read();
        let docs = self.documents.read();

        let scores: Vec<(usize, f32)> = vecs
            .iter()
            .enumerate()
            .map(|(i, v)| (i, cosine_similarity(&embedding, v)))
            .collect();

        let top_indices = top_k_by_score(scores, k);
        Ok(top_indices
            .into_iter()
            .filter_map(|i| docs.get(i).cloned())
            .collect())
    }

    async fn similarity_search_with_score(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        let embedding = self.embeddings.embed_query(query).await?;
        let vecs = self.vectors.read();
        let docs = self.documents.read();

        let scores: Vec<(usize, f32)> = vecs
            .iter()
            .enumerate()
            .map(|(i, v)| (i, cosine_similarity(&embedding, v)))
            .collect();

        let top_indices = top_k_by_score(scores, k);

        let results: Vec<(Document, f32)> = top_indices
            .into_iter()
            .filter_map(|i| {
                docs.get(i).map(|doc| {
                    let sim = cosine_similarity(&embedding, &vecs[i]);
                    (doc.clone(), 1.0 - sim)
                })
            })
            .collect();

        Ok(results)
    }

    async fn max_marginal_relevance_search(
        &self,
        query: &str,
        k: usize,
        fetch_k: usize,
        lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        let embedding = self.embeddings.embed_query(query).await?;
        let vecs = self.vectors.read();
        let docs = self.documents.read();

        let scores: Vec<(usize, f32)> = vecs
            .iter()
            .enumerate()
            .map(|(i, v)| (i, cosine_similarity(&embedding, v)))
            .collect();

        let fetch_indices: Vec<usize> = top_k_by_score(scores.clone(), fetch_k)
            .into_iter()
            .filter(|&i| i < docs.len())
            .collect();

        let fetch_embeddings: Vec<Vec<f32>> =
            fetch_indices.iter().map(|&i| vecs[i].clone()).collect();

        let mmr_indices =
            max_marginal_relevance(&embedding, &fetch_embeddings, k, lambda_mult);

        Ok(mmr_indices
            .into_iter()
            .map(|i| {
                let doc_idx = fetch_indices[i];
                docs[doc_idx].clone()
            })
            .collect())
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let mut docs_guard = self.documents.write();
        let mut vecs_guard = self.vectors.write();

        let id_set: std::collections::HashSet<String> = ids.into_iter().collect();
        let mut to_remove = Vec::new();

        for (i, doc) in docs_guard.iter().enumerate() {
            if let Some(serde_json::Value::String(ref id)) = doc.metadata.get("id") {
                if id_set.contains(id) {
                    to_remove.push(i);
                }
            }
        }

        to_remove.sort_by(|a, b| b.cmp(a));
        for &i in &to_remove {
            docs_guard.remove(i);
            vecs_guard.remove(i);
        }

        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
