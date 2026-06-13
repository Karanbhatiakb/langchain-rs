//! ArXiv paper search tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct ArxivTool {
    client: reqwest::Client,
}

impl Default for ArxivTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ArxivTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for ArxivTool {
    fn name(&self) -> &str {
        "arxiv"
    }

    fn description(&self) -> &str {
        "Arxiv API tool. Supports: search <query>, get_paper <arxiv_id>, download_paper <arxiv_id>. Uses the Arxiv API."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if let Some(cmd) = input.strip_prefix("search ") {
            let query = cmd.trim();
            let url = format!(
                "http://export.arxiv.org/api/query?search_query=all:{}&max_results=10&sortBy=relevance&sortOrder=descending",
                urlencode(query)
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Arxiv API error: {}", e)))?;

            let body = resp
                .text()
                .await
                .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?;

            let results = parse_atom_feed(&body);
            let mut output = Vec::new();
            for r in results {
                output.push(format!(
                    "{} - {} ({})",
                    r["id"].as_str().unwrap_or(""),
                    r["title"].as_str().unwrap_or(""),
                    r["published"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("get_paper ") {
            let paper_id = cmd.trim().trim_start_matches("arxiv:");
            let url = format!("http://export.arxiv.org/api/query?id_list={}", paper_id);
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Arxiv API error: {}", e)))?;

            let body = resp
                .text()
                .await
                .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?;

            let results = parse_atom_feed(&body);
            if let Some(paper) = results.into_iter().next() {
                return Ok(format!(
                    "Title: {}\nAuthors: {}\nPublished: {}\nSummary: {}",
                    paper["title"].as_str().unwrap_or(""),
                    paper["authors"].as_str().unwrap_or(""),
                    paper["published"].as_str().unwrap_or(""),
                    paper["summary"].as_str().unwrap_or(""),
                ));
            }
            return Err(ChainError::ToolError(format!("Paper not found: {}", paper_id)));
        }

        if let Some(cmd) = input.strip_prefix("download_paper ") {
            let paper_id = cmd.trim();
            let url = format!("https://arxiv.org/pdf/{}.pdf", paper_id);
            return Ok(format!("Download URL: {}", url));
        }

        Err(ChainError::ToolError(
            "Unknown Arxiv command. Supported: search, get_paper, download_paper".into(),
        ))
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

fn parse_atom_feed(xml: &str) -> Vec<serde_json::Value> {
    let mut results = Vec::new();

    for entry in xml.split("<entry>").skip(1) {
        let title = extract_tag(entry, "title").unwrap_or_default();
        let id = extract_tag(entry, "id").unwrap_or_default();
        let published = extract_tag(entry, "published").unwrap_or_default();
        let summary = extract_tag(entry, "summary").unwrap_or_default();

        let mut authors = Vec::new();
        for author_entry in entry.split("<author>").skip(1) {
            if let Some(name) = extract_tag(author_entry, "name") {
                authors.push(name);
            }
        }

        results.push(json!({
            "id": id,
            "title": title.replace('\n', " ").trim(),
            "published": published,
            "summary": summary.replace('\n', " ").trim(),
            "authors": authors.join(", "),
        }));
    }

    results
}

fn extract_tag(s: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    s.find(&open).and_then(|start| {
        let value_start = start + open.len();
        s[value_start..].find(&close).map(|end| {
            s[value_start..value_start + end].to_string()
        })
    })
}
