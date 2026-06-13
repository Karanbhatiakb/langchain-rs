//! HTML document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

pub struct HTMLLoader {
    file_path: String,
    css_selector: Option<String>,
}

impl HTMLLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            css_selector: None,
        }
    }

    pub fn with_css_selector(mut self, selector: impl Into<String>) -> Self {
        self.css_selector = Some(selector.into());
        self
    }
}

#[async_trait]
impl BaseLoader for HTMLLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read HTML file '{}': {}", self.file_path, e)))?;

        let text = if let Some(ref selector) = self.css_selector {
            extract_by_css_selector(&content, selector)
        } else {
            extract_text_from_html(&content)
        };

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("html".to_string()));

        Ok(vec![Document::new(text).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}

fn extract_text_from_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_name = String::new();

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            in_tag = true;
            tag_name.clear();
            i += 1;

            if i < chars.len() && chars[i] == '/' {
                // closing tag
                i += 1;
                while i < chars.len() && chars[i] != '>' {
                    tag_name.push(chars[i].to_ascii_lowercase());
                    i += 1;
                }
                if tag_name == "script" { in_script = false; }
                if tag_name == "style" { in_style = false; }

                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                continue;
            }

            while i < chars.len() && chars[i] != '>' && chars[i] != ' ' && chars[i] != '\n' && chars[i] != '\t' {
                tag_name.push(chars[i].to_ascii_lowercase());
                i += 1;
            }

            if tag_name == "script" { in_script = true; }
            if tag_name == "style" { in_style = true; }

            if tag_name == "br" || tag_name == "p" || tag_name == "div" || tag_name == "h1"
                || tag_name == "h2" || tag_name == "h3" || tag_name == "h4" || tag_name == "li" {
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
            }

            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
            continue;
        }

        if in_tag {
            if chars[i] == '>' {
                in_tag = false;
            }
            i += 1;
            continue;
        }

        if !in_script && !in_style {
            let c = chars[i];
            if c == '&' {
                let mut entity = String::new();
                i += 1;
                while i < chars.len() && chars[i] != ';' {
                    entity.push(chars[i]);
                    i += 1;
                }
                let decoded = match entity.as_str() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "nbsp" => " ",
                    _ => "",
                };
                result.push_str(decoded);
            } else {
                result.push(c);
            }
        }

        i += 1;
    }

    result.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_by_css_selector(html: &str, selector: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let selector = match Selector::parse(selector) {
        Ok(s) => s,
        Err(_) => return extract_text_from_html(html),
    };

    let mut texts = Vec::new();
    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        if !text.trim().is_empty() {
            texts.push(text.trim().to_string());
        }
    }

    if texts.is_empty() {
        return extract_text_from_html(html);
    }

    texts.join("\n")
}
