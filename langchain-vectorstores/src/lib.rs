//! Vector store implementations for similarity search and document storage.
//!
//! Provides the [`VectorStore`](traits::VectorStore) trait in [`traits`] and
//! implementations for Milvus, Chroma, FAISS, Pinecone, Qdrant, Weaviate,
//! pgvector, Redis, Elasticsearch, OpenSearch, Supabase, SingleStore, MongoDB,
//! Neo4j, LanceDB, DuckDB, and an in-memory store — each gated behind a
//! feature flag.

pub mod retrievers;
pub mod traits;
pub mod utils;
#[cfg(feature = "inmemory")] pub mod memory;
#[cfg(feature = "chroma")] pub mod chroma;
#[cfg(feature = "faiss")] pub mod faiss;
#[cfg(feature = "pinecone")] pub mod pinecone;
#[cfg(feature = "qdrant")] pub mod qdrant;
#[cfg(feature = "milvus")] pub mod milvus;
#[cfg(feature = "weaviate")] pub mod weaviate;
#[cfg(feature = "pgvector")] pub mod pgvector;
#[cfg(feature = "redis")] pub mod redis_store;
#[cfg(feature = "elasticsearch")] pub mod elasticsearch;
#[cfg(feature = "opensearch")] pub mod opensearch;
#[cfg(feature = "supabase")] pub mod supabase;
#[cfg(feature = "singlestore")] pub mod singlestore;
#[cfg(feature = "mongodb")] pub mod mongodb;
#[cfg(feature = "neo4j")] pub mod neo4j;
#[cfg(feature = "lancedb")] pub mod lancedb;
#[cfg(feature = "duckdb")] pub mod duckdb;
#[cfg(feature = "astra_db")] pub mod astra_db;
#[cfg(feature = "vectara")] pub mod vectara;
#[cfg(feature = "snowflake")] pub mod snowflake;
#[cfg(feature = "deep_lake")] pub mod deep_lake;
#[cfg(feature = "couchbase")] pub mod couchbase;
#[cfg(feature = "cratedb")] pub mod cratedb;
#[cfg(feature = "kdb_ai")] pub mod kdb_ai;
#[cfg(feature = "surrealdb")] pub mod surrealdb;
#[cfg(feature = "falkordb")] pub mod falkordb;
#[cfg(feature = "aerospike")] pub mod aerospike;
#[cfg(feature = "mysql_store")] pub mod mysql_store;
#[cfg(feature = "datastax")] pub mod datastax;

pub mod providers;
