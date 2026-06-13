//! YouTube audio transcript document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that fetches audio transcripts from YouTube videos.
///
/// Provide the video URL and an optional language code. The loader retrieves
/// the auto-generated or manual captions and returns segments as documents
/// with timestamp and language metadata.
#[derive(Debug, Clone)]
pub struct YoutubeAudioLoader {
    url: String,
    language: String,
}

impl YoutubeAudioLoader {
    /// Creates a new `YoutubeAudioLoader` for the given video URL and language.
    pub fn new(url: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            language: language.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for YoutubeAudioLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("youtube_audio".to_string()),
        );
        metadata.insert(
            "url".to_string(),
            serde_json::Value::String(self.url.clone()),
        );
        metadata.insert(
            "language".to_string(),
            serde_json::Value::String(self.language.clone()),
        );
        Ok(vec![Document::new("YouTube Audio loader stub — no transcript fetched yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
