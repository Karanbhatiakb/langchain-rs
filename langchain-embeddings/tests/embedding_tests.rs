use langchain_embeddings::fake::FakeEmbeddings;
use langchain_embeddings::traits::Embeddings;

#[cfg(feature = "voyage")]
use langchain_embeddings::voyage::VoyageEmbeddings;
#[cfg(feature = "nomic")]
use langchain_embeddings::nomic::NomicEmbeddings;
#[cfg(feature = "azure")]
use langchain_embeddings::azure::AzureEmbeddings;
#[cfg(feature = "deepseek")]
use langchain_embeddings::deepseek::DeepSeekEmbeddings;
#[cfg(feature = "gpt4all")]
use langchain_embeddings::gpt4all::GPT4AllEmbeddings;
#[cfg(feature = "llamacpp")]
use langchain_embeddings::llamacpp::LlamaCppEmbeddings;
#[cfg(feature = "minimax")]
use langchain_embeddings::minimax::MiniMaxEmbeddings;

#[tokio::test]
async fn test_fake_embeddings_embed_query() {
    let embeddings = FakeEmbeddings::new(10);
    let result = embeddings.embed_query("hello world").await.unwrap();
    assert_eq!(result.len(), 10);
    for val in &result {
        assert!(*val >= 0.0 && *val < 1.0);
    }
}

#[tokio::test]
async fn test_fake_embeddings_embed_documents() {
    let embeddings = FakeEmbeddings::new(5);
    let texts = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
    let result = embeddings.embed_documents(&texts).await.unwrap();
    assert_eq!(result.len(), 3);
    for embedding in &result {
        assert_eq!(embedding.len(), 5);
    }
}

#[tokio::test]
async fn test_fake_embeddings_deterministic() {
    let embeddings = FakeEmbeddings::new(8);
    let r1 = embeddings.embed_query("test query").await.unwrap();
    let r2 = embeddings.embed_query("test query").await.unwrap();
    assert_eq!(r1, r2);
}

#[tokio::test]
async fn test_fake_embeddings_different_texts_differ() {
    let embeddings = FakeEmbeddings::new(8);
    let r1 = embeddings.embed_query("hello").await.unwrap();
    let r2 = embeddings.embed_query("world").await.unwrap();
    assert_ne!(r1, r2);
}

#[tokio::test]
async fn test_fake_embeddings_dimension_1() {
    let embeddings = FakeEmbeddings::new(1);
    let result = embeddings.embed_query("test").await.unwrap();
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_fake_embeddings_dimension_large() {
    let embeddings = FakeEmbeddings::new(1536);
    let result = embeddings.embed_query("test").await.unwrap();
    assert_eq!(result.len(), 1536);
}

#[tokio::test]
async fn test_fake_embeddings_embed_documents_empty() {
    let embeddings = FakeEmbeddings::new(5);
    let result = embeddings.embed_documents(&[]).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_fake_embeddings_embed_documents_single() {
    let embeddings = FakeEmbeddings::new(3);
    let texts = vec!["one".to_string()];
    let result = embeddings.embed_documents(&texts).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].len(), 3);
}

#[cfg(feature = "voyage")]
mod voyage_tests {
    use super::*;
    #[test]
    fn test_voyage_embeddings_creation() {
        let emb = VoyageEmbeddings::new("test-key");
        assert_eq!(emb.embedding_dimension(), 1024);
    }
    #[test]
    fn test_voyage_embeddings_with_model() {
        let emb = VoyageEmbeddings::new("test-key").with_model("voyage-3");
        assert_eq!(emb.embedding_dimension(), 1024);
    }
}

#[cfg(feature = "nomic")]
mod nomic_tests {
    use super::*;
    #[test]
    fn test_nomic_embeddings_creation() {
        let emb = NomicEmbeddings::new("test-key");
        assert_eq!(emb.embedding_dimension(), 768);
    }
    #[test]
    fn test_nomic_embeddings_with_model() {
        let emb = NomicEmbeddings::new("test-key").with_model("nomic-embed-text");
        assert_eq!(emb.embedding_dimension(), 768);
    }
}

#[cfg(feature = "azure")]
mod azure_tests {
    use super::*;
    #[test]
    fn test_azure_embeddings_creation() {
        let emb = AzureEmbeddings::new("test-key", "my-resource", "my-deployment");
        assert_eq!(emb.embedding_dimension(), 1536);
    }
    #[test]
    fn test_azure_embeddings_with_api_version() {
        let emb = AzureEmbeddings::new("test-key", "my-resource", "my-deployment")
            .with_api_version("2024-02-01");
        assert_eq!(emb.embedding_dimension(), 1536);
    }
}

#[cfg(feature = "deepseek")]
mod deepseek_tests {
    use super::*;
    #[test]
    fn test_deepseek_embeddings_creation() {
        let emb = DeepSeekEmbeddings::new("test-key");
        assert_eq!(emb.embedding_dimension(), 1024);
    }
    #[test]
    fn test_deepseek_embeddings_with_model() {
        let emb = DeepSeekEmbeddings::new("test-key").with_model("deepseek-embedding");
        assert_eq!(emb.embedding_dimension(), 1024);
    }
}

#[cfg(feature = "gpt4all")]
mod gpt4all_tests {
    use super::*;
    #[test]
    fn test_gpt4all_embeddings_creation() {
        let emb = GPT4AllEmbeddings::new();
        assert_eq!(emb.embedding_dimension(), 768);
    }
    #[test]
    fn test_gpt4all_embeddings_with_base_url() {
        let emb = GPT4AllEmbeddings::new().with_base_url("http://localhost:4891");
        assert_eq!(emb.embedding_dimension(), 768);
    }
}

#[cfg(feature = "llamacpp")]
mod llamacpp_tests {
    use super::*;
    #[test]
    fn test_llamacpp_embeddings_creation() {
        let emb = LlamaCppEmbeddings::new();
        assert_eq!(emb.embedding_dimension(), 4096);
    }
    #[test]
    fn test_llamacpp_embeddings_with_base_url() {
        let emb = LlamaCppEmbeddings::new().with_base_url("http://localhost:8080");
        assert_eq!(emb.embedding_dimension(), 4096);
    }
}

#[cfg(feature = "minimax")]
mod minimax_tests {
    use super::*;
    #[test]
    fn test_minimax_embeddings_creation() {
        let emb = MiniMaxEmbeddings::new("test-key", "group-1");
        assert_eq!(emb.embedding_dimension(), 2560);
    }
    #[test]
    fn test_minimax_embeddings_with_model() {
        let emb = MiniMaxEmbeddings::new("test-key", "group-1").with_model("embo-01");
        assert_eq!(emb.embedding_dimension(), 2560);
    }
}
