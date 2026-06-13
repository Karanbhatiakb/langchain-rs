//! Core traits, types, and utilities for the LangChain framework.
//!
//! This crate provides foundational building blocks including:
//! - Error types and result aliases [`errors`]
//! - Message types for chat interactions [`messages`]
//! - Document representation [`documents`]
//! - Utility functions [`utils`]
//! - Runnable trait and LCEL primitives [`runnable`], [`lcel`]
//! - Prompt templating [`prompt`]
//! - Output parsers [`output_parsers`]
//! - Callback handlers [`callbacks`]
//! - Caching layers [`caches`]
//! - Key-value stores [`stores`]
//! - Retrievers [`retrievers`]
//! - Example selectors [`example_selectors`]
//! - Text splitters [`text_splitters`]
//! - Tokenization [`tokenization`]
//! - Generation/chat schemas [`schemas`]
//! - Streaming primitives [`streaming`]
//! - Runnable configuration [`config`]
//! - Embedding model trait and test fakes [`embeddings`]
//! - Vector store trait and in-memory implementation [`vectorstores`]

pub mod errors;
pub mod messages;
pub mod documents;
pub mod utils;
pub mod runnable;
pub mod lcel;
pub mod prompt;
pub mod output_parsers;
pub mod callbacks;
pub mod caches;
pub mod stores;
pub mod retrievers;
pub mod example_selectors;
pub mod doc_stores;
pub mod runnables;
pub mod schema;
pub mod text_splitters;
pub mod tokenization;
pub mod schemas;
pub mod streaming;
pub mod config;
pub mod rate_limiters;
pub mod chat_loaders;
pub mod cross_encoders;
pub mod structured_query;
pub mod indexing;
pub mod compressors;
pub mod embeddings;
pub mod language_models;
pub mod prompt_values;
pub mod agents;
pub mod outputs;
pub mod load;
pub mod globals;
pub mod chat_history;
pub mod document_transformers;
pub mod tools;
pub mod tracers;
pub mod env;
pub mod sys_info;
