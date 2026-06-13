use std::collections::HashMap;
use std::sync::Arc;

use langchain_core::documents::Document;
use langchain_embeddings::fake::FakeEmbeddings;
use langchain_vectorstores::memory::InMemoryVectorStore;
use langchain_vectorstores::traits::VectorStore;

#[cfg(feature = "astra_db")]
use langchain_vectorstores::astra_db::AstraDBVectorStore;
#[cfg(feature = "vectara")]
use langchain_vectorstores::vectara::VectaraVectorStore;
#[cfg(feature = "snowflake")]
use langchain_vectorstores::snowflake::SnowflakeVectorStore;
#[cfg(feature = "deep_lake")]
use langchain_vectorstores::deep_lake::DeepLakeVectorStore;
#[cfg(feature = "couchbase")]
use langchain_vectorstores::couchbase::CouchbaseVectorStore;
#[cfg(feature = "cratedb")]
use langchain_vectorstores::cratedb::CrateDBVectorStore;
#[cfg(feature = "kdb_ai")]
use langchain_vectorstores::kdb_ai::KDBAIVectorStore;
#[cfg(feature = "surrealdb")]
use langchain_vectorstores::surrealdb::SurrealDBVectorStore;
#[cfg(feature = "falkordb")]
use langchain_vectorstores::falkordb::FalkorDBVectorStore;
#[cfg(feature = "aerospike")]
use langchain_vectorstores::aerospike::AerospikeVectorStore;
#[cfg(feature = "mysql_store")]
use langchain_vectorstores::mysql_store::MySQLVectorStore;
#[cfg(feature = "datastax")]
use langchain_vectorstores::datastax::DataStaxVectorStore;

fn create_store(dim: usize) -> InMemoryVectorStore {
    InMemoryVectorStore::new(Arc::new(FakeEmbeddings::new(dim)))
}

#[tokio::test]
async fn test_add_texts() {
    let store = create_store(5);
    let ids = store.add_texts(
        vec!["hello".to_string(), "world".to_string()],
        None,
    ).await.unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids[0].starts_with("doc_"));
}

#[tokio::test]
async fn test_add_texts_with_metadata() {
    let store = create_store(5);
    let metadatas = Some(vec![
        {
            let mut m = HashMap::new();
            m.insert("key".to_string(), serde_json::Value::String("val1".to_string()));
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("key".to_string(), serde_json::Value::String("val2".to_string()));
            m
        },
    ]);
    let ids = store.add_texts(
        vec!["doc1".to_string(), "doc2".to_string()],
        metadatas,
    ).await.unwrap();
    assert_eq!(ids.len(), 2);
}

#[tokio::test]
async fn test_add_documents() {
    let store = create_store(5);
    let docs = vec![
        Document::new("doc one"),
        Document::new("doc two"),
    ];
    let ids = store.add_documents(docs).await.unwrap();
    assert_eq!(ids.len(), 2);
}

#[tokio::test]
async fn test_similarity_search() {
    let store = create_store(5);
    store.add_texts(
        vec!["cat".to_string(), "dog".to_string(), "bird".to_string()],
        None,
    ).await.unwrap();
    let results = store.similarity_search("cat", 2).await.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_similarity_search_by_vector() {
    let store = create_store(5);
    store.add_texts(
        vec!["alpha".to_string(), "beta".to_string()],
        None,
    ).await.unwrap();
    let embedding = store.embeddings().embed_query("alpha").await.unwrap();
    let results = store.similarity_search_by_vector(embedding, 1).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_similarity_search_with_score() {
    let store = create_store(5);
    store.add_texts(
        vec!["alpha".to_string(), "beta".to_string()],
        None,
    ).await.unwrap();
    let results = store.similarity_search_with_score("alpha", 2).await.unwrap();
    assert_eq!(results.len(), 2);
    let (_doc, score) = &results[0];
    assert!(*score >= 0.0);
}

#[tokio::test]
async fn test_max_marginal_relevance_search() {
    let store = create_store(5);
    store.add_texts(
        vec![
            "cat".to_string(),
            "dog".to_string(),
            "bird".to_string(),
            "fish".to_string(),
        ],
        None,
    ).await.unwrap();
    let results = store.max_marginal_relevance_search("cat", 2, 4, 0.5).await.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_similarity_search_k_greater_than_docs() {
    let store = create_store(5);
    store.add_texts(
        vec!["only one".to_string()],
        None,
    ).await.unwrap();
    let results = store.similarity_search("only one", 10).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_add_texts_empty() {
    let store = create_store(5);
    let ids = store.add_texts(vec![], None).await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn test_sequential_adds() {
    let store = create_store(5);
    let ids1 = store.add_texts(vec!["first".to_string()], None).await.unwrap();
    let ids2 = store.add_texts(vec!["second".to_string()], None).await.unwrap();
    assert_eq!(ids1.len(), 1);
    assert_eq!(ids2.len(), 1);
    let results = store.similarity_search("first", 2).await.unwrap();
    assert_eq!(results.len(), 2);
}

#[cfg(feature = "astra_db")]
mod astra_db_tests {
    use super::*;
    fn create_astra_store() -> AstraDBVectorStore {
        AstraDBVectorStore::new(
            "db-id",
            "us-east1",
            "keyspace1",
            "collection1",
            "test-token",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_astra_db_store_creation() {
        let _store = create_astra_store();
    }
}

#[cfg(feature = "vectara")]
mod vectara_tests {
    use super::*;
    fn create_vectara_store() -> VectaraVectorStore {
        VectaraVectorStore::new(
            "cust-1",
            "corp-1",
            "test-key",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_vectara_store_creation() {
        let _store = create_vectara_store();
    }
}

#[cfg(feature = "snowflake")]
mod snowflake_tests {
    use super::*;
    fn create_snowflake_store() -> SnowflakeVectorStore {
        SnowflakeVectorStore::new(
            "account",
            "db",
            "schema",
            "table",
            "user",
            "pass",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_snowflake_store_creation() {
        let _store = create_snowflake_store();
    }
    #[test]
    fn test_snowflake_store_with_warehouse() {
        let store = create_snowflake_store().with_warehouse("wh");
        let _ = &store;
    }
    #[test]
    fn test_snowflake_store_with_role() {
        let store = create_snowflake_store().with_role("admin");
        let _ = &store;
    }
}

#[cfg(feature = "deep_lake")]
mod deep_lake_tests {
    use super::*;
    fn create_deep_lake_store() -> DeepLakeVectorStore {
        DeepLakeVectorStore::new("dataset/path", Arc::new(FakeEmbeddings::new(5)))
    }
    #[test]
    fn test_deep_lake_store_creation() {
        let _store = create_deep_lake_store();
    }
    #[test]
    fn test_deep_lake_store_with_api_key() {
        let store = create_deep_lake_store().with_api_key("test-key");
        let _ = &store;
    }
    #[test]
    fn test_deep_lake_store_with_overwrite() {
        let store = create_deep_lake_store().with_overwrite(true);
        let _ = &store;
    }
}

#[cfg(feature = "couchbase")]
mod couchbase_tests {
    use super::*;
    fn create_couchbase_store() -> CouchbaseVectorStore {
        CouchbaseVectorStore::new(
            "http://localhost:8091",
            "bucket",
            "scope",
            "collection",
            "user",
            "pass",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_couchbase_store_creation() {
        let _store = create_couchbase_store();
    }
    #[test]
    fn test_couchbase_store_with_index_name() {
        let store = create_couchbase_store().with_index_name("my_index");
        let _ = &store;
    }
}

#[cfg(feature = "cratedb")]
mod cratedb_tests {
    use super::*;
    fn create_cratedb_store() -> CrateDBVectorStore {
        CrateDBVectorStore::new(
            "localhost",
            4200,
            "doc",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_cratedb_store_creation() {
        let _store = create_cratedb_store();
    }
    #[test]
    fn test_cratedb_store_with_credentials() {
        let store = create_cratedb_store().with_credentials("user", "pass");
        let _ = &store;
    }
}

#[cfg(feature = "kdb_ai")]
mod kdb_ai_tests {
    use super::*;
    fn create_kdb_ai_store() -> KDBAIVectorStore {
        KDBAIVectorStore::new(
            "http://localhost:8080",
            "test-key",
            "table1",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_kdb_ai_store_creation() {
        let _store = create_kdb_ai_store();
    }
}

#[cfg(feature = "surrealdb")]
mod surrealdb_tests {
    use super::*;
    fn create_surrealdb_store() -> SurrealDBVectorStore {
        SurrealDBVectorStore::new(
            "http://localhost:8000",
            "ns",
            "db",
            "table1",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_surrealdb_store_creation() {
        let _store = create_surrealdb_store();
    }
    #[test]
    fn test_surrealdb_store_with_credentials() {
        let store = create_surrealdb_store().with_credentials("user", "pass");
        let _ = &store;
    }
    #[test]
    fn test_surrealdb_store_with_token() {
        let store = create_surrealdb_store().with_token("token123");
        let _ = &store;
    }
}

#[cfg(feature = "falkordb")]
mod falkordb_tests {
    use super::*;
    fn create_falkordb_store() -> FalkorDBVectorStore {
        FalkorDBVectorStore::new(
            "localhost",
            6379,
            "graph1",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_falkordb_store_creation() {
        let _store = create_falkordb_store();
    }
    #[test]
    fn test_falkordb_store_with_password() {
        let store = create_falkordb_store().with_password("secret");
        let _ = &store;
    }
}

#[cfg(feature = "aerospike")]
mod aerospike_tests {
    use super::*;
    fn create_aerospike_store() -> AerospikeVectorStore {
        AerospikeVectorStore::new(
            "localhost",
            3000,
            "ns1",
            "set1",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_aerospike_store_creation() {
        let _store = create_aerospike_store();
    }
    #[test]
    fn test_aerospike_store_with_credentials() {
        let store = create_aerospike_store().with_credentials("user", "pass");
        let _ = &store;
    }
    #[test]
    fn test_aerospike_store_with_index_name() {
        let store = create_aerospike_store().with_index_name("my_index");
        let _ = &store;
    }
}

#[cfg(feature = "mysql_store")]
mod mysql_store_tests {
    use super::*;
    fn create_mysql_store() -> MySQLVectorStore {
        MySQLVectorStore::new(
            "localhost",
            3306,
            "db",
            "table1",
            "user",
            "pass",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_mysql_store_creation() {
        let _store = create_mysql_store();
    }
}

#[cfg(feature = "datastax")]
mod datastax_tests {
    use super::*;
    fn create_datastax_store() -> DataStaxVectorStore {
        DataStaxVectorStore::new(
            "localhost",
            9042,
            "keyspace1",
            "table1",
            Arc::new(FakeEmbeddings::new(5)),
        )
    }
    #[test]
    fn test_datastax_store_creation() {
        let _store = create_datastax_store();
    }
    #[test]
    fn test_datastax_store_with_credentials() {
        let store = create_datastax_store().with_credentials("user", "pass");
        let _ = &store;
    }
    #[test]
    fn test_datastax_store_with_secure_connect_bundle() {
        let store = create_datastax_store().with_secure_connect_bundle("/path/to/bundle");
        let _ = &store;
    }
}
