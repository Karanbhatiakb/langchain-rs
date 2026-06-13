//! vector_store_demo module.

use std::sync::Arc;

use langchain_core::documents::Document;
use langchain_vectorstores::traits::VectorStore;

pub async fn run() -> anyhow::Result<()> {
    println!("InMemoryVectorStore with FakeEmbeddings:");

    #[derive(Clone)]
    struct FakeEmbeddings;
    #[async_trait::async_trait]
    impl langchain_embeddings::traits::Embeddings for FakeEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> langchain_core::errors::Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| {
            let dim = 4;
            let hash: u32 = t.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
            (0..dim).map(|i| ((hash >> (i * 8)) & 0xFF) as f32 / 255.0).collect()
        }).collect())
    }
    async fn embed_query(&self, text: &str) -> langchain_core::errors::Result<Vec<f32>> {
        self.embed_documents(&[text.to_string()]).await.map(|v| v.into_iter().next().unwrap_or_default())
        }
    }

    let embeddings = Arc::new(FakeEmbeddings);
    let store = langchain_vectorstores::memory::InMemoryVectorStore::new(embeddings);

    store.add_documents(vec![
        Document::new("Rust is a systems programming language"),
        Document::new("Python is a high-level programming language"),
        Document::new("JavaScript runs in the browser"),
        Document::new("Rust focuses on safety and performance"),
    ]).await?;

    let results = store.similarity_search("Rust", 2).await?;
    println!("  Similarity search for 'Rust':");
    for doc in &results {
        println!("    - {}", doc.page_content);
    }

    let results_with_score = store.similarity_search_with_score("programming", 3).await?;
    println!("  Similarity search with score for 'programming':");
    for (doc, score) in &results_with_score {
        println!("    [{:.4}] {}", score, doc.page_content);
    }

    Ok(())
}
