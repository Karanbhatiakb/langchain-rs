//! GitHub repository document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

#[derive(Debug, Deserialize, Serialize)]
struct GitHubContent {
    #[serde(rename = "type")]
    content_type: String,
    name: String,
    path: String,
    download_url: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPR {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    html_url: String,
}

pub struct GitHubLoader {
    owner: String,
    repo: String,
    branch: String,
    client: Client,
    token: String,
}

impl GitHubLoader {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        let token = std::env::var("GITHUB_TOKEN")
            .unwrap_or_default();
        Self {
            owner: owner.into(),
            repo: repo.into(),
            branch: "main".to_string(),
            client: Client::new(),
            token,
        }
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = branch.into();
        self
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = token.into();
        self
    }

    fn auth_value(&self) -> Option<String> {
        if self.token.is_empty() {
            None
        } else {
            Some(format!("Bearer {}", self.token))
        }
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let url = format!("https://api.github.com{}", endpoint);
        let mut req = self.client.get(&url)
            .header("User-Agent", "langchain-rs/0.1")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(auth) = self.auth_value() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await
            .map_err(|e| ChainError::IOError(format!("GitHub API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "GitHub API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read GitHub API response: {}", e)))
    }

    pub async fn list_contents(&self, path: &str) -> Result<Vec<Document>> {
        let endpoint = format!("/repos/{}/{}/contents/{}?ref={}", self.owner, self.repo, path, self.branch);
        let body = self.api_get(&endpoint).await?;

        let contents: Vec<GitHubContent> = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse GitHub contents: {}", e)))?;

        let mut documents = Vec::new();
        for item in contents {
            let content_str = serde_json::to_string(&item)
                .unwrap_or_default();
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(item.path.clone()));
            metadata.insert("name".to_string(), serde_json::Value::String(item.name));
            metadata.insert("content_type".to_string(), serde_json::Value::String(item.content_type));
            metadata.insert("repo".to_string(), serde_json::Value::String(format!("{}/{}", self.owner, self.repo)));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("github".to_string()));
            documents.push(Document::new(content_str).with_metadata(metadata));
        }

        Ok(documents)
    }

    pub async fn get_file(&self, path: &str) -> Result<Document> {
        let endpoint = format!("/repos/{}/{}/contents/{}?ref={}", self.owner, self.repo, path, self.branch);
        let body = self.api_get(&endpoint).await?;

        let content: GitHubContent = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse GitHub file: {}", e)))?;

        let decoded = content.content
            .as_ref()
            .and_then(|c| {
                let cleaned = c.replace('\n', "").replace('\r', "");
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.decode(cleaned.as_bytes()).ok()
            })
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(content.path.clone()));
        metadata.insert("name".to_string(), serde_json::Value::String(content.name));
        metadata.insert("repo".to_string(), serde_json::Value::String(format!("{}/{}", self.owner, self.repo)));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("github".to_string()));

        Ok(Document::new(decoded).with_metadata(metadata))
    }

    pub async fn get_repo_issues(&self, state: Option<&str>) -> Result<Vec<Document>> {
        let state_param = state.unwrap_or("open");
        let endpoint = format!("/repos/{}/{}/issues?state={}", self.owner, self.repo, state_param);
        let body = self.api_get(&endpoint).await?;

        let issues: Vec<GitHubIssue> = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse GitHub issues: {}", e)))?;

        let mut documents = Vec::new();
        for issue in issues {
            let content = issue.body.clone().unwrap_or_default();
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(issue.html_url));
            metadata.insert("title".to_string(), serde_json::Value::String(issue.title));
            metadata.insert("number".to_string(), serde_json::Value::Number(issue.number.into()));
            metadata.insert("state".to_string(), serde_json::Value::String(issue.state));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("github_issues".to_string()));
            documents.push(Document::new(content).with_metadata(metadata));
        }

        Ok(documents)
    }

    pub async fn get_prs(&self, state: Option<&str>) -> Result<Vec<Document>> {
        let state_param = state.unwrap_or("open");
        let endpoint = format!("/repos/{}/{}/pulls?state={}", self.owner, self.repo, state_param);
        let body = self.api_get(&endpoint).await?;

        let prs: Vec<GitHubPR> = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse GitHub PRs: {}", e)))?;

        let mut documents = Vec::new();
        for pr in prs {
            let content = pr.body.clone().unwrap_or_default();
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(pr.html_url));
            metadata.insert("title".to_string(), serde_json::Value::String(pr.title));
            metadata.insert("number".to_string(), serde_json::Value::Number(pr.number.into()));
            metadata.insert("state".to_string(), serde_json::Value::String(pr.state));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("github_pr".to_string()));
            documents.push(Document::new(content).with_metadata(metadata));
        }

        Ok(documents)
    }
}

#[async_trait]
impl BaseLoader for GitHubLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.list_contents("").await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
