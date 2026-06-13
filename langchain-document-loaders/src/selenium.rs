//! Selenium URL document loader.
//!
//! Uses a Selenium WebDriver (via HTTP) to load a page and retrieve its
//! rendered HTML.  Requires a running Selenium server.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads a web page via Selenium WebDriver for JavaScript-rendered content.
#[derive(Debug, Clone)]
pub struct SeleniumLoader {
    url: String,
    selenium_url: String,
    client: Client,
}

impl SeleniumLoader {
    /// Create a new `SeleniumLoader`.
    ///
    /// `selenium_url` defaults to `http://localhost:4444/wd/hub` if not set.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            selenium_url: "http://localhost:4444/wd/hub".to_string(),
            client: Client::new(),
        }
    }

    /// Override the Selenium WebDriver URL.
    pub fn with_selenium_url(mut self, selenium_url: impl Into<String>) -> Self {
        self.selenium_url = selenium_url.into();
        self
    }
}

#[async_trait]
impl BaseLoader for SeleniumLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let session_payload = serde_json::json!({
            "capabilities": {
                "alwaysMatch": {
                    "browserName": "chrome"
                }
            }
        });

        let session_resp = self.client
            .post(format!("{}/session", self.selenium_url))
            .json(&session_payload)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Selenium session creation failed: {}", e)))?;

        let session_body: serde_json::Value = session_resp.json().await
            .map_err(|e| ChainError::IOError(format!("Failed to parse Selenium session response: {}", e)))?;

        let session_id = session_body["value"]["sessionId"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ChainError::IOError("Selenium did not return a session ID".to_string()))?;

        let navigate_payload = serde_json::json!({ "url": self.url });
        self.client
            .post(format!("{}/session/{}/url", self.selenium_url, session_id))
            .json(&navigate_payload)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Selenium navigation failed: {}", e)))?;

        let page_resp = self.client
            .get(format!("{}/session/{}/source", self.selenium_url, session_id))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Selenium page source request failed: {}", e)))?;

        let page_body: serde_json::Value = page_resp.json().await
            .map_err(|e| ChainError::IOError(format!("Failed to parse Selenium page source: {}", e)))?;

        let html = page_body["value"]
            .as_str()
            .unwrap_or("");

        self.client
            .delete(format!("{}/session/{}", self.selenium_url, session_id))
            .send()
            .await
            .ok();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.url.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("selenium".to_string()));

        Ok(vec![Document::new(html.to_string()).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
