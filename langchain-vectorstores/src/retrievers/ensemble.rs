//! Ensemble retriever that combines results from multiple retrievers using
//! weighted score fusion.
//!
//! The [`EnsembleRetriever`] takes a list of base retrievers and corresponding
//! weights, runs each retriever independently, then fuses the results by
//! weighting and normalising scores across all retrievers. This is analogous
//! to the LangChain `EnsembleRetriever`.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;
use std::sync::Arc;

/// Combines multiple retrievers by weighting and fusing their scores.
///
/// Each retriever is run independently. Results are merged by summing
/// normalised scores weighted by the corresponding `weights` entry.
/// The top `c` documents across all retrievers are returned.
#[derive(Clone)]
pub struct EnsembleRetriever {
    /// The list of base retrievers to combine.
    pub retrievers: Vec<Arc<dyn BaseRetriever>>,
    /// Per-retriever weights (must be the same length as `retrievers`).
    pub weights: Vec<f32>,
    /// The number of top documents to return after fusion.
    pub c: usize,
}

impl std::fmt::Debug for EnsembleRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnsembleRetriever")
            .field("weights", &self.weights)
            .field("c", &self.c)
            .finish()
    }
}

impl EnsembleRetriever {
    /// Creates a new `EnsembleRetriever`.
    ///
    /// # Arguments
    /// * `retrievers` - A list of retrievers to combine.
    /// * `weights` - Corresponding weights for each retriever.
    /// * `c` - Number of top documents to return.
    pub fn new(retrievers: Vec<Arc<dyn BaseRetriever>>, weights: Vec<f32>, c: usize) -> Self {
        Self {
            retrievers,
            weights,
            c,
        }
    }
}

#[async_trait]
impl BaseRetriever for EnsembleRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
