//! PubMed document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

const PUBMED_EUTILS: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";

pub struct PubMedLoader {
    query: String,
    max_results: u32,
    client: Client,
}

impl PubMedLoader {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            max_results: 10,
            client: Client::new(),
        }
    }

    pub fn with_max_results(mut self, max: u32) -> Self {
        self.max_results = max;
        self
    }

    async fn esearch(&self) -> Result<Vec<String>> {
        let encoded = url::form_urlencoded::byte_serialize(self.query.as_bytes()).collect::<String>();
        let url = format!(
            "{}/esearch.fcgi?db=pubmed&term={}&retmax={}&retmode=json",
            PUBMED_EUTILS, encoded, self.max_results
        );

        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("PubMed ESearch failed: {}", e)))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| ChainError::ParserError(format!("Failed to parse ESearch response: {}", e)))?;

        let ids = body.pointer("/esearchresult/idlist")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ids)
    }

    async fn efetch(&self, ids: &[String]) -> Result<String> {
        let id_list = ids.join(",");
        let url = format!(
            "{}/efetch.fcgi?db=pubmed&id={}&retmode=xml&rettype=abstract",
            PUBMED_EUTILS, id_list
        );

        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("PubMed EFetch failed: {}", e)))?;

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read PubMed EFetch response: {}", e)))
    }

    fn parse_pubmed_response(xml: &str) -> Vec<Document> {
        let mut documents = Vec::new();

        for article_xml in xml.split("<PubmedArticle>").skip(1) {
            let pmid = extract_xml_tag_simple(article_xml, "PMID")
                .or_else(|| extract_xml_tag_simple(article_xml, "ArticleId"))
                .unwrap_or_default();
            let title = extract_xml_tag_simple(article_xml, "ArticleTitle")
                .unwrap_or_default();
            let abstract_text = extract_xml_tag_simple(article_xml, "AbstractText")
                .unwrap_or_default();
            let journal = extract_xml_tag_simple(article_xml, "Title")
                .unwrap_or_default();
            let pub_date = extract_xml_tag_simple(article_xml, "PubDate")
                .unwrap_or_default();

            let content = format!("Title: {}\nJournal: {}\nDate: {}\n\nAbstract: {}", title, journal, pub_date, abstract_text);

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(format!("pubmed:{}", pmid)));
            metadata.insert("pmid".to_string(), serde_json::Value::String(pmid));
            metadata.insert("title".to_string(), serde_json::Value::String(title));
            metadata.insert("journal".to_string(), serde_json::Value::String(journal));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("pubmed".to_string()));

            documents.push(Document::new(content).with_metadata(metadata));
        }

        documents
    }
}

fn extract_xml_tag_simple(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = content.find(&open) {
        let value_start = start + open.len();
        if let Some(end) = content[value_start..].find(&close) {
            return Some(content[value_start..value_start + end].trim().to_string());
        }
    }
    None
}

#[async_trait]
impl BaseLoader for PubMedLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let ids = self.esearch().await?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let xml = self.efetch(&ids).await?;
        Ok(Self::parse_pubmed_response(&xml))
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
