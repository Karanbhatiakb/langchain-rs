//! YouTube transcript document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct YouTubeLoader {
    video_id: String,
    api_key: String,
    client: Client,
}

impl YouTubeLoader {
    pub fn new(video_id: impl Into<String>) -> Self {
        let api_key = std::env::var("YOUTUBE_API_KEY")
            .expect("YOUTUBE_API_KEY environment variable is required");
        Self {
            video_id: video_id.into(),
            api_key,
            client: Client::new(),
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    pub fn video_id(&self) -> &str {
        &self.video_id
    }

    pub async fn get_video_metadata(&self) -> Result<Document> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?id={}&key={}&part=snippet,statistics",
            self.video_id, self.api_key
        );

        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("YouTube API request failed: {}", e)))?;

        let body: Value = response.json().await
            .map_err(|e| ChainError::ParserError(format!("Failed to parse YouTube API response: {}", e)))?;

        let snippet = body.pointer("/items/0/snippet")
            .ok_or_else(|| ChainError::IOError(format!("Video '{}' not found", self.video_id)))?;

        let title = snippet.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let description = snippet.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let channel = snippet.get("channelTitle").and_then(|v| v.as_str()).unwrap_or("");

        let content = format!("Title: {}\nChannel: {}\nDescription: {}", title, channel, description);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(
            format!("https://www.youtube.com/watch?v={}", self.video_id)
        ));
        metadata.insert("video_id".to_string(), serde_json::Value::String(self.video_id.clone()));
        metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
        metadata.insert("channel".to_string(), serde_json::Value::String(channel.to_string()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("youtube".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }

    pub async fn get_transcript(&self) -> Result<Document> {
        let captions_url = format!(
            "https://www.googleapis.com/youtube/v3/captions?videoId={}&key={}&part=snippet",
            self.video_id, self.api_key
        );

        let response = self.client.get(&captions_url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("YouTube captions request failed: {}", e)))?;

        let body: Value = response.json().await
            .map_err(|e| ChainError::ParserError(format!("Failed to parse captions response: {}", e)))?;

        let mut content = String::new();
        let title = "YouTube Transcript".to_string();

        if let Some(items) = body.get("items").and_then(|i| i.as_array()) {
            for item in items {
                if let Some(caption_id) = item.get("id").and_then(|v| v.as_str()) {
                    let track_url = format!(
                        "https://www.googleapis.com/youtube/v3/captions/{}?key={}&fmt=srt",
                        caption_id, self.api_key
                    );
                    if let Ok(track_resp) = self.client.get(&track_url).send().await {
                        if let Ok(text) = track_resp.text().await {
                            content.push_str(&text);
                            content.push('\n');
                        }
                    }
                }
            }
        }

        if content.is_empty() {
            content = "No captions available for this video.".to_string();
        }

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(
            format!("https://www.youtube.com/watch?v={}", self.video_id)
        ));
        metadata.insert("video_id".to_string(), serde_json::Value::String(self.video_id.clone()));
        metadata.insert("title".to_string(), serde_json::Value::String(title));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("youtube_transcript".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }
}

#[async_trait]
impl BaseLoader for YouTubeLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let metadata_doc = self.get_video_metadata().await?;
        let transcript_doc = self.get_transcript().await?;
        Ok(vec![metadata_doc, transcript_doc])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
