//! Retriever implementations for document retrieval from various sources.
//!
//! This module provides a collection of retrievers that implement the
//! [`BaseRetriever`] trait, including ensemble methods, algorithm-based
//! retrievers (BM25, TF-IDF, SVM, KNN), web search retrievers (Wikipedia,
//! Arxiv, PubMed, Tavily), and composite retrievers (multi-query,
//! contextual compression, parent document, self-query, etc.).
//!
//! Each retriever is behind a default feature flag and exposed unconditionally
//! when the crate is built.

pub mod ensemble;
pub mod merger;
pub mod multi_vector;
pub mod rephrase;
pub mod self_query;
pub mod time_weighted;
pub mod bm25;
pub mod svm;
pub mod tf_idf;
pub mod knn;
pub mod wikipedia;
pub mod arxiv;
pub mod pubmed;
pub mod tavily;
pub mod web_research;
pub mod parent_document;
pub mod multi_query;
pub mod contextual;
pub mod kendra;
pub mod elasticsearch;

pub use ensemble::EnsembleRetriever;
pub use merger::MergerRetriever;
pub use multi_vector::MultiVectorRetriever;
pub use rephrase::RePhraseQueryRetriever;
pub use self_query::SelfQueryRetriever;
pub use time_weighted::TimeWeightedVectorStoreRetriever;
pub use bm25::BM25Retriever;
pub use svm::SVMRetriever;
pub use tf_idf::TFIDFRetriever;
pub use knn::KNNRetriever;
pub use wikipedia::WikipediaRetriever;
pub use arxiv::ArxivRetriever;
pub use pubmed::PubMedRetriever;
pub use tavily::TavilySearchAPIRetriever;
pub use web_research::WebResearchRetriever;
pub use parent_document::ParentDocumentRetriever;
pub use multi_query::MultiQueryRetriever;
pub use contextual::ContextualCompressionRetriever;
pub use kendra::AmazonKendraRetriever;
pub use elasticsearch::ElasticSearchRetriever;
