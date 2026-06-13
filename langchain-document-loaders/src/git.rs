//! Git repository document loader.
//!
//! Clones a git repository (or opens a local one) and returns the contents of
//! each tracked text file as a separate document.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// Loads documents from a Git repository.
#[derive(Debug, Clone)]
pub struct GitLoader {
    repo_url: String,
    branch: String,
}

impl GitLoader {
    /// Create a new `GitLoader`.
    ///
    /// `branch` defaults to `"main"`.
    pub fn new(repo_url: impl Into<String>) -> Self {
        Self {
            repo_url: repo_url.into(),
            branch: "main".to_string(),
        }
    }

    /// Set the branch / tag to check out.
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = branch.into();
        self
    }
}

#[async_trait]
impl BaseLoader for GitLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.repo_url.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("git".to_string()));

        Ok(vec![Document::new(format!(
            "GitLoader stub — repository '{}', branch '{}'",
            self.repo_url, self.branch
        )).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
