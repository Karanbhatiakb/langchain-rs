//! Embedding model providers — convert text into vector representations.
//!
//! Defines the [`Embeddings`](traits::Embeddings) trait in [`traits`] and
//! provides implementations for OpenAI, HuggingFace, Cohere, Google, Bedrock,
//! Mistral, Jina, and Ollama — each gated behind a feature flag.

pub mod traits;
pub mod fake;

#[cfg(feature = "openai")]
pub mod openai;
#[cfg(feature = "huggingface")]
pub mod huggingface;
#[cfg(feature = "cohere")]
pub mod cohere;
#[cfg(feature = "google")]
pub mod google;
#[cfg(feature = "bedrock")]
pub mod bedrock;
#[cfg(feature = "mistral")]
pub mod mistral;
#[cfg(feature = "jina")]
pub mod jina;
#[cfg(feature = "ollama")]
pub mod ollama;
#[cfg(feature = "voyage")]
pub mod voyage;
#[cfg(feature = "nomic")]
pub mod nomic;
#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "deepseek")]
pub mod deepseek;
#[cfg(feature = "gpt4all")]
pub mod gpt4all;
#[cfg(feature = "llamacpp")]
pub mod llamacpp;
#[cfg(feature = "minimax")]
pub mod minimax;

pub mod providers;

#[cfg(test)]
mod tests {
    use super::traits::Embeddings;
    use super::fake::FakeEmbeddings;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_fake_embeddings_construct() {
        let emb = FakeEmbeddings::new(4);
        let result = emb.embed_query("hello").await.unwrap();
        assert_eq!(result.len(), 4);
    }

    #[tokio::test]
    async fn test_fake_embeddings_documents() {
        let emb = FakeEmbeddings::new(3);
        let docs = emb.embed_documents(&["a".into(), "b".into()]).await.unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].len(), 3);
    }

    #[tokio::test]
    async fn test_fake_embeddings_empty_documents() {
        let emb = FakeEmbeddings::new(5);
        let docs = emb.embed_documents(&[]).await.unwrap();
        assert!(docs.is_empty());
    }

    #[tokio::test]
    async fn test_trait_object() {
        let emb: Arc<dyn Embeddings> = Arc::new(FakeEmbeddings::new(2));
        let v = emb.embed_query("test").await.unwrap();
        assert_eq!(v.len(), 2);
    }

    #[tokio::test]
    async fn test_embeddings_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeEmbeddings>();
        assert_sync::<FakeEmbeddings>();
    }
}
