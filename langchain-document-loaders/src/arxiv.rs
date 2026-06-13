//! ArXiv document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

const ARXIV_API: &str = "https://export.arxiv.org/api/query";

pub struct ArxivLoader {
    query: String,
    id_list: Vec<String>,
    max_results: u32,
    client: Client,
}

impl ArxivLoader {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            id_list: Vec::new(),
            max_results: 10,
            client: Client::new(),
        }
    }

    pub fn with_max_results(mut self, max: u32) -> Self {
        self.max_results = max;
        self
    }

    pub fn with_ids(mut self, ids: Vec<String>) -> Self {
        self.id_list = ids;
        self
    }

    async fn query_api(&self) -> Result<String> {
        let encoded = url::form_urlencoded::byte_serialize(self.query.as_bytes()).collect::<String>();
        let url = if self.id_list.is_empty() {
            format!("{}?search_query={}&max_results={}", ARXIV_API, encoded, self.max_results)
        } else {
            format!("{}?id_list={}", ARXIV_API, self.id_list.join(","))
        };

        let response = self.client.get(&url)
            .header("User-Agent", "langchain-rs/0.1")
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Arxiv API request failed: {}", e)))?;

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Arxiv API response: {}", e)))
    }

    fn parse_arxiv_response(xml: &str) -> Vec<Document> {
        let mut documents = Vec::new();

        for entry in xml.split("<entry>").skip(1) {
            let id = extract_xml_tag(entry, "id").unwrap_or_default();
            let title = extract_xml_tag(entry, "title")
                .map(|t| t.replace('\n', " ").trim().to_string())
                .unwrap_or_default();
            let summary = extract_xml_tag(entry, "summary")
                .map(|s| s.replace('\n', " ").trim().to_string())
                .unwrap_or_default();
            let published = extract_xml_tag(entry, "published").unwrap_or_default();

            let authors: Vec<String> = entry.split("<author>")
                .skip(1)
                .filter_map(|a| extract_xml_tag(a, "name"))
                .collect();

            let pdf_link = entry.split("<link")
                .skip(1)
                .filter_map(|l| {
                    if l.contains("title=\"pdf\"") {
                        l.split("href=\"")
                            .nth(1)
                            .and_then(|h| h.split('\"').next())
                            .map(String::from)
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or_default();

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(id.clone()));
            metadata.insert("title".to_string(), serde_json::Value::String(title));
            metadata.insert("authors".to_string(), serde_json::Value::String(authors.join(", ")));
            metadata.insert("published".to_string(), serde_json::Value::String(published));
            metadata.insert("pdf_url".to_string(), serde_json::Value::String(pdf_link));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("arxiv".to_string()));

            documents.push(Document::new(summary).with_metadata(metadata));
        }

        documents
    }
}

fn extract_xml_tag(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = content.find(&open) {
        let value_start = start + open.len();
        if let Some(end) = content[value_start..].find(&close) {
            return Some(content[value_start..value_start + end].to_string());
        }
    }
    let open_attr = format!("<{} ", tag);
    if let Some(start) = content.find(&open_attr) {
        if let Some(close_tag) = content[start..].find('>') {
            let value_start = start + close_tag + 1;
            if let Some(end) = content[value_start..].find(&close) {
                return Some(content[value_start..value_start + end].to_string());
            }
        }
    }
    None
}

#[async_trait]
impl BaseLoader for ArxivLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let xml = self.query_api().await?;
        Ok(Self::parse_arxiv_response(&xml))
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
