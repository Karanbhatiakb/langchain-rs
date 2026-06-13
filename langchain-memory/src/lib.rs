//! Memory implementations for storing and retrieving conversation history.
//!
//! Provides the [`BaseMemory`](traits::BaseMemory) trait and implementations
//! for buffer, window, summary, entity, vector store, Zep, Postgres, Redis,
//! MongoDB, Cassandra, DynamoDB, SQLite, and dozens more — each gated behind a
//! feature flag.

pub mod traits;
pub mod chat_message_histories;
pub mod combined;
pub mod prompt_memory;
pub mod readonly;
pub mod token_buffer;
pub mod vectorstore_token_buffer;
#[cfg(feature = "buffer")] pub mod buffer;
#[cfg(feature = "window")] pub mod window;
#[cfg(feature = "summary")] pub mod summary;
#[cfg(feature = "entity")] pub mod entity;
#[cfg(feature = "vectorstore")] pub mod vector_store_memory;
#[cfg(feature = "zep")] pub mod zep_memory;
#[cfg(feature = "postgres")] pub mod postgres;
#[cfg(feature = "redis")] pub mod redis_memory;
#[cfg(feature = "mongodb")] pub mod mongodb_memory;
#[cfg(feature = "cassandra")] pub mod cassandra;
#[cfg(feature = "dynamodb")] pub mod dynamodb;
#[cfg(feature = "sqlite")] pub mod sqlite_memory;
#[cfg(feature = "motorhead")] pub mod motorhead;
#[cfg(feature = "memgraph")] pub mod memgraph;
#[cfg(feature = "postgres-history")] pub mod postgres_history;
#[cfg(feature = "redis-history")] pub mod redis_history;
#[cfg(feature = "mongodb-history")] pub mod mongodb_history;
#[cfg(feature = "dynamodb-history")] pub mod dynamodb_history;
#[cfg(feature = "elasticsearch-history")] pub mod elasticsearch_history;
#[cfg(feature = "firestore-history")] pub mod firestore_history;
#[cfg(feature = "cosmosdb-history")] pub mod cosmosdb_history;
#[cfg(feature = "sql-history")] pub mod sql_history;
#[cfg(feature = "neo4j-history")] pub mod neo4j_history;
#[cfg(feature = "astra-history")] pub mod astra_history;
