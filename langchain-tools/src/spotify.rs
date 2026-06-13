//! Spotify API tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct SpotifyTool {
    access_token: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for SpotifyTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyTool {
    pub fn new() -> Self {
        let access_token = std::env::var("SPOTIFY_ACCESS_TOKEN").unwrap_or_default();
        Self {
            access_token,
            base_url: "https://api.spotify.com/v1".into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_access_token(mut self, token: &str) -> Self {
        self.access_token = token.to_string();
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[async_trait]
impl BaseTool for SpotifyTool {
    fn name(&self) -> &str {
        "spotify"
    }

    fn description(&self) -> &str {
        "Spotify API tool. Supports: search <query>, play <track_uri>, pause, current_track, top_tracks, top_artists."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty Spotify command".into()));
        }
        if self.access_token.is_empty() {
            return Err(ChainError::ToolError("SPOTIFY_ACCESS_TOKEN not set".into()));
        }

        if let Some(query) = input.strip_prefix("search ") {
            let url = format!(
                "{}/search?q={}&type=track,artist,album&limit=5",
                self.base_url,
                urlencode(query.trim())
            );
            return self.get_json(&url).await;
        }

        if let Some(uri) = input.strip_prefix("play ") {
            let url = format!("{}/me/player/play", self.base_url);
            let body = serde_json::json!({ "uris": [uri.trim()] });
            return self.put_json(&url, &body).await;
        }

        if input == "pause" {
            let url = format!("{}/me/player/pause", self.base_url);
            return self.put_json(&url, &serde_json::json!({})).await;
        }

        if input == "current_track" {
            let url = format!("{}/me/player/currently-playing", self.base_url);
            return self.get_json(&url).await;
        }

        if input == "top_tracks" {
            let url = format!("{}/me/top/tracks?limit=5", self.base_url);
            return self.get_json(&url).await;
        }

        if input == "top_artists" {
            let url = format!("{}/me/top/artists?limit=5", self.base_url);
            return self.get_json(&url).await;
        }

        Err(ChainError::ToolError(
            "Unknown Spotify command. Supported: search, play, pause, current_track, top_tracks, top_artists".into(),
        ))
    }
}

impl SpotifyTool {
    async fn get_json(&self, url: &str) -> ToolResult {
        let resp = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Spotify API error: {}", e)))?;

        if !resp.status().is_success() && resp.status().as_u16() != 204 {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Spotify API returned {}: {}",
                status, text
            )));
        }

        let text = resp.text().await.map_err(|e| ChainError::ToolError(format!("Read error: {}", e)))?;
        if text.is_empty() {
            return Ok("No content".into());
        }
        let result: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
    }

    async fn put_json(&self, url: &str, body: &serde_json::Value) -> ToolResult {
        let resp = self
            .client
            .put(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Spotify API error: {}", e)))?;

        if !resp.status().is_success() && resp.status().as_u16() != 204 {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Spotify API returned {}: {}",
                status, text
            )));
        }

        Ok("OK".into())
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => '+'.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
