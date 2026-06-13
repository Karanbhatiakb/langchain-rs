//! Reddit document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::sync::Mutex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct RedditLoader {
    client_id: String,
    client_secret: String,
    user_agent: String,
    subreddit: Option<String>,
    client: Client,
    access_token: Arc<Mutex<Option<String>>>,
}

impl RedditLoader {
    pub fn new() -> Self {
        let client_id = std::env::var("REDDIT_CLIENT_ID")
            .expect("REDDIT_CLIENT_ID environment variable is required");
        let client_secret = std::env::var("REDDIT_CLIENT_SECRET")
            .expect("REDDIT_CLIENT_SECRET environment variable is required");
        let user_agent = std::env::var("REDDIT_USER_AGENT")
            .unwrap_or_else(|_| "langchain-rs/0.1".to_string());

        Self {
            client_id,
            client_secret,
            user_agent,
            subreddit: None,
            client: Client::new(),
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_subreddit(mut self, subreddit: impl Into<String>) -> Self {
        self.subreddit = Some(subreddit.into());
        self
    }

    async fn authenticate(&self) -> Result<String> {
        {
            let token = self.access_token.lock().unwrap();
            if token.is_some() {
                return Ok(token.clone().unwrap());
            }
        }

        let body = "grant_type=client_credentials";
        let auth = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.client_id, self.client_secret),
        );

        let response = self.client.post("https://www.reddit.com/api/v1/access_token")
            .header("Authorization", format!("Basic {}", auth))
            .header("User-Agent", &self.user_agent)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Reddit auth failed: {}", e)))?;

        let value: Value = response.json().await
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Reddit auth response: {}", e)))?;

        let token = value.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ChainError::IOError("Failed to get Reddit access token".to_string()))?
            .to_string();

        *self.access_token.lock().unwrap() = Some(token.clone());
        Ok(token)
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let token = self.authenticate().await?;
        let url = format!("https://oauth.reddit.com{}", endpoint);

        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Reddit API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Reddit API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Reddit API response: {}", e)))
    }

    pub async fn get_subreddit_posts(&self, subreddit: Option<&str>, limit: Option<u32>) -> Result<Vec<Document>> {
        let sr = subreddit.or(self.subreddit.as_deref())
            .ok_or_else(|| ChainError::ValidationError("No subreddit provided".to_string()))?;
        let limit = limit.unwrap_or(25);

        let body = self.api_get(&format!("/r/{}/hot?limit={}", sr, limit)).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Reddit posts: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(children) = value.pointer("/data/children").and_then(|c| c.as_array()) {
            for child in children {
                if let Some(data) = child.get("data") {
                    let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let selftext = data.get("selftext").and_then(|v| v.as_str()).unwrap_or("");
                    let author = data.get("author").and_then(|v| v.as_str()).unwrap_or("");
                    let permalink = data.get("permalink").and_then(|v| v.as_str()).unwrap_or("");
                    let score = data.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
                    let num_comments = data.get("num_comments").and_then(|v| v.as_i64()).unwrap_or(0);

                    let content = format!("Title: {}\nAuthor: {}\nScore: {}\nComments: {}\n\n{}", title, author, score, num_comments, selftext);

                    let mut metadata = HashMap::new();
                    metadata.insert("source".to_string(), serde_json::Value::String(
                        format!("https://www.reddit.com{}", permalink)
                    ));
                    metadata.insert("subreddit".to_string(), serde_json::Value::String(sr.to_string()));
                    metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
                    metadata.insert("author".to_string(), serde_json::Value::String(author.to_string()));
                    metadata.insert("score".to_string(), serde_json::Value::Number(score.into()));
                    metadata.insert("loader_type".to_string(), serde_json::Value::String("reddit".to_string()));

                    documents.push(Document::new(content).with_metadata(metadata));
                }
            }
        }

        Ok(documents)
    }

    pub async fn get_post_comments(&self, post_id: &str) -> Result<Vec<Document>> {
        let body = self.api_get(&format!("/comments/{}?limit=100&depth=1", post_id)).await?;
        let values: Vec<Value> = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Reddit comments: {}", e)))?;

        let mut documents = Vec::new();
        if values.len() > 1 {
            if let Some(comments) = values[1].pointer("/data/children").and_then(|c| c.as_array()) {
                for child in comments {
                    if let Some(data) = child.get("data") {
                        let body_text = data.get("body").and_then(|v| v.as_str()).unwrap_or("");
                        let author = data.get("author").and_then(|v| v.as_str()).unwrap_or("");
                        let score = data.get("score").and_then(|v| v.as_i64()).unwrap_or(0);

                        let mut metadata = HashMap::new();
                        metadata.insert("source".to_string(), serde_json::Value::String(format!("reddit_comment:{}", post_id)));
                        metadata.insert("author".to_string(), serde_json::Value::String(author.to_string()));
                        metadata.insert("post_id".to_string(), serde_json::Value::String(post_id.to_string()));
                        metadata.insert("score".to_string(), serde_json::Value::Number(score.into()));
                        metadata.insert("loader_type".to_string(), serde_json::Value::String("reddit".to_string()));

                        documents.push(Document::new(body_text.to_string()).with_metadata(metadata));
                    }
                }
            }
        }

        Ok(documents)
    }
}

impl Default for RedditLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for RedditLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let sr = self.subreddit.clone()
            .unwrap_or_else(|| "rust".to_string());
        self.get_subreddit_posts(Some(&sr), None).await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        let sr = self.subreddit.clone().unwrap_or_else(|| "rust".to_string());
        let result = RedditLoader {
            subreddit: Some(sr),
            ..RedditLoader::new()
        }.get_subreddit_posts(None, None).await;
        match result {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
