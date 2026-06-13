//! Directory document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use walkdir::WalkDir;

use crate::traits::BaseLoader;

pub type LoaderFactory = Arc<dyn Send + Sync + Fn(&str) -> Result<Box<dyn BaseLoader>>>;

pub struct DirectoryLoader {
    path: String,
    glob: Option<String>,
    recursive: bool,
    hidden_files: bool,
    max_depth: Option<usize>,
    loader_factory: Option<LoaderFactory>,
}

impl DirectoryLoader {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            glob: None,
            recursive: true,
            hidden_files: false,
            max_depth: None,
            loader_factory: None,
        }
    }

    pub fn with_glob(mut self, glob: impl Into<String>) -> Self {
        self.glob = Some(glob.into());
        self
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn with_hidden_files(mut self, hidden_files: bool) -> Self {
        self.hidden_files = hidden_files;
        self
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn with_loader_factory(mut self, factory: LoaderFactory) -> Self {
        self.loader_factory = Some(factory);
        self
    }
}

fn extension_matches(path: &str, glob_pattern: &Option<String>) -> bool {
    match glob_pattern {
        Some(pattern) => {
            if pattern.starts_with("*.") {
                let ext = &pattern[1..];
                path.ends_with(ext)
            } else {
                if let Ok(g) = glob::Pattern::new(pattern) {
                    g.matches(path)
                } else {
                    true
                }
            }
        }
        None => true,
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

#[async_trait]
impl BaseLoader for DirectoryLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut walker = WalkDir::new(&self.path);
        walker = walker.follow_links(true);

        if !self.recursive {
            walker = walker.max_depth(1);
        }
        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        let mut all_docs = Vec::new();

        for entry in walker {
            let entry = entry
                .map_err(|e| ChainError::IOError(format!("Directory walk error: {}", e)))?;

            if !entry.file_type().is_file() {
                continue;
            }

            if !self.hidden_files && is_hidden(&entry) {
                continue;
            }

            let path_str = entry.path().to_string_lossy().to_string();

            if !extension_matches(&path_str, &self.glob) {
                continue;
            }

            if let Some(ref factory) = self.loader_factory {
                let loader = factory(&path_str)
                    .map_err(|e| ChainError::IOError(format!("Factory error for {}: {}", path_str, e)))?;
                let docs = loader.load().await?;
                all_docs.extend(docs);
            } else {
                let content = tokio::fs::read_to_string(&path_str).await
                    .unwrap_or_else(|_| String::new());

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(path_str));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("directory".to_string()));

                all_docs.push(Document::new(content).with_metadata(metadata));
            }
        }

        Ok(all_docs)
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
