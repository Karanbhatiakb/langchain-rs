//! Obsidian vault document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

use crate::traits::BaseLoader;

pub struct ObsidianLoader {
    vault_path: String,
    recursive: bool,
}

impl ObsidianLoader {
    pub fn new(vault_path: impl Into<String>) -> Self {
        Self {
            vault_path: vault_path.into(),
            recursive: true,
        }
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
}

#[async_trait]
impl BaseLoader for ObsidianLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        let mut walker = WalkDir::new(&self.vault_path);
        walker = walker.follow_links(true);

        if !self.recursive {
            walker = walker.max_depth(1);
        }

        let vault_path = self.vault_path.clone();

        let entries: Vec<_> = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("md"))
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();

        for file_path in entries {
            let content = fs::read_to_string(&file_path)
                .await
                .map_err(|e| ChainError::IOError(format!("Failed to read Obsidian file '{}': {}", file_path, e)))?;

            let relative_path = Path::new(&file_path)
                .strip_prefix(&vault_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file_path.clone());

            let frontmatter = extract_frontmatter(&content);

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(file_path.clone()));
            metadata.insert("relative_path".to_string(), serde_json::Value::String(relative_path));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("obsidian".to_string()));

            if let Some(fm) = frontmatter {
                metadata.insert("frontmatter".to_string(), serde_json::Value::String(fm));
            }

            let clean_content = strip_frontmatter(&content);

            documents.push(Document::new(clean_content).with_metadata(metadata));
        }

        Ok(documents)
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}

fn extract_frontmatter(content: &str) -> Option<String> {
    let content = content.trim();
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            return Some(content[3..3 + end].trim().to_string());
        }
    }
    None
}

fn strip_frontmatter(content: &str) -> String {
    let content = content.trim();
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            return content[3 + end + 3..].trim().to_string();
        }
    }
    content.to_string()
}
