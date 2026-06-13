//! Wikipedia document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct WikipediaLoader {
    query: String,
    lang: String,
    client: Client,
}

impl WikipediaLoader {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            lang: "en".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    fn api_url(&self) -> String {
        format!("https://{}.wikipedia.org/w/api.php", self.lang)
    }

    async fn api_get(&self, params: &[(&str, &str)]) -> Result<String> {
        let url = self.api_url();
        let response = self.client.get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Wikipedia API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Wikipedia API returned HTTP {}", response.status()
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Wikipedia API response: {}", e)))
    }

    pub async fn search(&self, query: Option<&str>) -> Result<Vec<Document>> {
        let q = query.unwrap_or(&self.query);
        let params = &[
            ("action", "query"),
            ("list", "search"),
            ("srsearch", q),
            ("format", "json"),
            ("srlimit", "50"),
        ];

        let body = self.api_get(params).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Wikipedia search results: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(results) = value.pointer("/query/search").and_then(|r| r.as_array()) {
            for result in results {
                let title = result.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let snippet = result.get("snippet").and_then(|v| v.as_str()).unwrap_or("");

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(
                    format!("https://{}.wikipedia.org/wiki/{}", self.lang, title.replace(' ', "_"))
                ));
                metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("wikipedia".to_string()));

                documents.push(Document::new(
                    format!("Title: {}\nSnippet: {}", title, html_cleaner(&snippet))
                ).with_metadata(metadata));
            }
        }

        Ok(documents)
    }

    pub async fn summary(&self, title: Option<&str>) -> Result<Document> {
        let t = title.unwrap_or(&self.query);
        let params = &[
            ("action", "query"),
            ("prop", "extracts"),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("titles", t),
            ("format", "json"),
        ];

        let body = self.api_get(params).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Wikipedia summary: {}", e)))?;

        let pages = value.pointer("/query/pages").and_then(|p| p.as_object()).cloned().unwrap_or_default();
        let content = pages.values().next()
            .and_then(|page| page.get("extract").and_then(|v| v.as_str()))
            .unwrap_or("Page not found")
            .to_string();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(
            format!("https://{}.wikipedia.org/wiki/{}", self.lang, t.replace(' ', "_"))
        ));
        metadata.insert("title".to_string(), serde_json::Value::String(t.to_string()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("wikipedia".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }

    pub async fn full_page(&self, title: Option<&str>) -> Result<Document> {
        let t = title.unwrap_or(&self.query);
        let params = &[
            ("action", "query"),
            ("prop", "extracts"),
            ("explaintext", "1"),
            ("titles", t),
            ("format", "json"),
        ];

        let body = self.api_get(params).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Wikipedia page: {}", e)))?;

        let pages = value.pointer("/query/pages").and_then(|p| p.as_object()).cloned().unwrap_or_default();
        let content = pages.values().next()
            .and_then(|page| page.get("extract").and_then(|v| v.as_str()))
            .unwrap_or("Page not found")
            .to_string();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(
            format!("https://{}.wikipedia.org/wiki/{}", self.lang, t.replace(' ', "_"))
        ));
        metadata.insert("title".to_string(), serde_json::Value::String(t.to_string()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("wikipedia".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }

    pub async fn random(&self) -> Result<Document> {
        let params = &[
            ("action", "query"),
            ("list", "random"),
            ("rnlimit", "1"),
            ("rnnamespace", "0"),
            ("format", "json"),
        ];

        let body = self.api_get(params).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Wikipedia random: {}", e)))?;

        let title = value.pointer("/query/random/0/title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        self.full_page(Some(&title)).await
    }

    pub async fn category_members(&self, category: &str, limit: Option<u32>) -> Result<Vec<Document>> {
        let limit = limit.unwrap_or(50);
        let params = &[
            ("action", "query"),
            ("list", "categorymembers"),
            ("cmtitle", &format!("Category:{}", category)),
            ("cmlimit", &limit.to_string()),
            ("format", "json"),
        ];

        let body = self.api_get(params).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Wikipedia category: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(members) = value.pointer("/query/categorymembers").and_then(|m| m.as_array()) {
            for member in members {
                let title = member.get("title").and_then(|v| v.as_str()).unwrap_or("");

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(
                    format!("https://{}.wikipedia.org/wiki/{}", self.lang, title.replace(' ', "_"))
                ));
                metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
                metadata.insert("category".to_string(), serde_json::Value::String(category.to_string()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("wikipedia".to_string()));

                documents.push(Document::new(title.to_string()).with_metadata(metadata));
            }
        }

        Ok(documents)
    }
}

fn html_cleaner(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}

#[async_trait]
impl BaseLoader for WikipediaLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.search(None).await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
