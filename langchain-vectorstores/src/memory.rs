//! In-memory vector store implementation.

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

pub struct InMemoryVectorStore {
    documents: Arc<RwLock<Vec<Document>>>,
    embeddings: Arc<dyn Embeddings>,
    vectors: Arc<RwLock<Vec<Vec<f32>>>>,
}

impl InMemoryVectorStore {
    pub fn new(embeddings: Arc<dyn Embeddings>) -> Self {
        Self {
            documents: Arc::new(RwLock::new(Vec::new())),
            embeddings,
            vectors: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        let docs: Vec<Document> = texts.iter().enumerate().map(|(i, text)| {
            let mut doc = Document::new(text.clone());
            if let Some(ref metas) = metadatas {
                if let Some(meta) = metas.get(i) {
                    doc.metadata = meta.clone();
                }
            }
            doc
        }).collect();

        let embeddings = self.embeddings.embed_documents(&texts).await?;

        let mut docs_guard = self.documents.write();
        let mut vecs_guard = self.vectors.write();

        let ids: Vec<String> = docs
            .iter()
            .enumerate()
            .map(|(i, _)| format!("doc_{}", docs_guard.len() + i))
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
            .map(|(i, _)| format!("doc_{}", docs_guard.len() + i))
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
        Ok(top_indices.into_iter().map(|i| docs[i].clone()).collect())
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
            .map(|i| {
                let sim = cosine_similarity(&embedding, &vecs[i]);
                (docs[i].clone(), 1.0 - sim)
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

        let fetch_indices: Vec<usize> = top_k_by_score(scores, fetch_k)
            .into_iter()
            .filter(|&i| i < docs.len())
            .collect();

        let fetch_embeddings: Vec<Vec<f32>> =
            fetch_indices.iter().map(|&i| vecs[i].clone()).collect();

        let mmr_indices = max_marginal_relevance(&embedding, &fetch_embeddings, k, lambda_mult);

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

#[cfg(test)]
mod tests {
    use super::*;
    use langchain_embeddings::fake::FakeEmbeddings;

    fn make_store() -> InMemoryVectorStore {
        InMemoryVectorStore::new(Arc::new(FakeEmbeddings::new(4)))
    }

    #[tokio::test]
    async fn test_add_texts_and_search() {
        let store = make_store();
        let ids = store.add_texts(vec!["hello world".to_string(), "foo bar".to_string()], None).await.unwrap();
        assert_eq!(ids.len(), 2);
        let results = store.similarity_search("hello", 1).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_similarity_search_with_score() {
        let store = make_store();
        store.add_texts(vec!["document one".to_string(), "document two".to_string()], None).await.unwrap();
        let results = store.similarity_search_with_score("document", 2).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].1 >= 0.0);
    }

    #[tokio::test]
    async fn test_search_by_vector() {
        let store = make_store();
        store.add_texts(vec!["test content".to_string()], None).await.unwrap();
        let query_vec = vec![0.5; 4];
        let results = store.similarity_search_by_vector(query_vec, 1).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_max_marginal_relevance_search() {
        let store = make_store();
        store.add_texts(vec!["doc a".to_string(), "doc b".to_string(), "doc c".to_string()], None).await.unwrap();
        let results = store.max_marginal_relevance_search("doc", 2, 3, 0.5).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_by_id() {
        let store = make_store();
        let ids = store.add_texts(vec!["to delete".to_string()], None).await.unwrap();
        store.delete(ids).await.unwrap();
        let results = store.similarity_search("delete", 1).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_add_documents() {
        let store = make_store();
        let docs = vec![Document::new("hello"), Document::new("world")];
        let ids = store.add_documents(docs).await.unwrap();
        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn test_search_empty_store() {
        let store = make_store();
        let results = store.similarity_search("anything", 5).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_embeddings_getter() {
        let store = make_store();
        let emb = store.embeddings();
        let v = emb.embed_query("test").await.unwrap();
        assert_eq!(v.len(), 4);
    }

    #[tokio::test]
    async fn test_in_memory_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<InMemoryVectorStore>();
        assert_sync::<InMemoryVectorStore>();
    }
}
