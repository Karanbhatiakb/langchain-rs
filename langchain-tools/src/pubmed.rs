//! PubMed article search tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct PubMedTool {
    client: reqwest::Client,
}

impl Default for PubMedTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PubMedTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for PubMedTool {
    fn name(&self) -> &str {
        "pubmed"
    }

    fn description(&self) -> &str {
        "PubMed API tool using NCBI E-utilities. Supports: search <query>, fetch_abstract <pmid>, fetch_paper <pmid>. Uses the NCBI PubMed API."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if let Some(cmd) = input.strip_prefix("search ") {
            let query = cmd.trim();
            let url = format!(
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term={}&retmax=10&retmode=json",
                urlencode(query)
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("PubMed API error: {}", e)))?;

            let result: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

            let ids = result["esearchresult"]["idlist"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let mut output = Vec::new();
            for id in ids {
                output.push(id.as_str().unwrap_or("").to_string());
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("fetch_abstract ") {
            let pmid = cmd.trim();
            let url = format!(
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&id={}&retmode=xml&rettype=abstract",
                pmid
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("PubMed API error: {}", e)))?;

            let body = resp
                .text()
                .await
                .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?;

            let abstract_text = extract_abstract(&body);
            return Ok(abstract_text);
        }

        if let Some(cmd) = input.strip_prefix("fetch_paper ") {
            let pmid = cmd.trim();
            let url = format!(
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&id={}&retmode=xml&rettype=abstract",
                pmid
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("PubMed API error: {}", e)))?;

            let body = resp
                .text()
                .await
                .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?;

            let title = extract_tag(&body, "ArticleTitle").unwrap_or_default();
            let abstract_text = extract_abstract(&body);
            let authors = extract_tag(&body, "Author").unwrap_or_default();

            return Ok(format!(
                "Title: {}\nAuthors: {}\nAbstract: {}",
                title, authors, abstract_text
            ));
        }

        Err(ChainError::ToolError(
            "Unknown PubMed command. Supported: search, fetch_abstract, fetch_paper".into(),
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

fn extract_abstract(xml: &str) -> String {
    let mut parts = Vec::new();
    let mut remaining = xml;

    while let Some(start) = remaining.find("<AbstractText") {
        let text_start = remaining[start..].find('>').map(|i| start + i + 1);
        let text_end = remaining[start..].find("</AbstractText>").map(|i| start + i);

        if let (Some(ts), Some(te)) = (text_start, text_end) {
            let text = &remaining[ts..te];
            parts.push(text.to_string());
            remaining = &remaining[te + 14..];
        } else {
            break;
        }
    }

    parts.join("\n")
}

fn extract_tag<'a>(s: &'a str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    s.find(&open).and_then(|start| {
        let value_start = start + open.len();
        s[value_start..].find(&close).map(|end| {
            s[value_start..value_start + end].to_string()
        })
    })
}
