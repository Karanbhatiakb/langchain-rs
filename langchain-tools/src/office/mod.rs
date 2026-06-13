//! Office productivity tool implementations.

pub use crate::notion::NotionTool;
pub use crate::github::GitHubTool;
pub use crate::google_calendar::GoogleCalendarTool;

use async_trait::async_trait;
use crate::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct JiraTool {
    base_url: String,
    client: reqwest::Client,
    email: String,
    api_token: String,
}

impl JiraTool {
    pub fn new(base_url: impl Into<String>, email: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            email: email.into(),
            api_token: api_token.into(),
        }
    }
}

#[async_trait]
impl BaseTool for JiraTool {
    fn name(&self) -> &str {
        "jira"
    }

    fn description(&self) -> &str {
        "Interact with Jira. Supports: search <JQL>, get_issue <key>, create_issue <project>\\n<summary>\\n<description>\\n<type>"
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        let url_base = format!("{}/rest/api/2", self.base_url);

        if let Some(jql) = input.strip_prefix("search ") {
            let url = format!("{}/search?jql={}&maxResults=5", url_base, urlencode(jql));
            let resp = self.client.get(&url)
                .basic_auth(&self.email, Some(&self.api_token))
                .send().await
                .map_err(|e| ChainError::ToolError(format!("Jira API error: {}", e)))?;
            let result: serde_json::Value = resp.json().await
                .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let mut output = Vec::new();
            if let Some(issues) = result["issues"].as_array() {
                for issue in issues.iter().take(5) {
                    let key = issue["key"].as_str().unwrap_or("");
                    let summary = issue["fields"]["summary"].as_str().unwrap_or("");
                    let status = issue["fields"]["status"]["name"].as_str().unwrap_or("");
                    output.push(format!("{} - [{}] {}", key, status, summary));
                }
            }
            Ok(output.join("\n"))
        } else if let Some(key) = input.strip_prefix("get_issue ") {
            let url = format!("{}/issue/{}", url_base, key.trim());
            let resp = self.client.get(&url)
                .basic_auth(&self.email, Some(&self.api_token))
                .send().await
                .map_err(|e| ChainError::ToolError(format!("Jira API error: {}", e)))?;
            let result: serde_json::Value = resp.json().await
                .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let summary = result["fields"]["summary"].as_str().unwrap_or("");
            let status = result["fields"]["status"]["name"].as_str().unwrap_or("");
            let desc = result["fields"]["description"].as_str().unwrap_or("None");
            Ok(format!("{} - [{}]\n{}\n\n{}", key.trim(), status, summary, desc))
        } else if let Some(rest) = input.strip_prefix("create_issue ") {
            let parts: Vec<&str> = rest.splitn(4, '\n').collect();
            if parts.len() < 4 {
                return Err(ChainError::ToolError("Usage: create_issue project\\nsummary\\ndescription\\ntype".into()));
            }
            let body = serde_json::json!({
                "fields": {
                    "project": { "key": parts[0].trim() },
                    "summary": parts[1].trim(),
                    "description": parts[2].trim(),
                    "issuetype": { "name": parts[3].trim() }
                }
            });
            let url = format!("{}/issue", url_base);
            let resp = self.client.post(&url)
                .basic_auth(&self.email, Some(&self.api_token))
                .json(&body)
                .send().await
                .map_err(|e| ChainError::ToolError(format!("Jira API error: {}", e)))?;
            let result: serde_json::Value = resp.json().await
                .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let key = result["key"].as_str().unwrap_or("unknown");
            Ok(format!("Created issue: {}", key))
        } else {
            Err(ChainError::ToolError("Unknown Jira command. Use: search, get_issue, create_issue".into()))
        }
    }
}

pub struct ConfluenceTool {
    base_url: String,
    client: reqwest::Client,
    email: String,
    api_token: String,
}

impl ConfluenceTool {
    pub fn new(base_url: impl Into<String>, email: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            email: email.into(),
            api_token: api_token.into(),
        }
    }
}

#[async_trait]
impl BaseTool for ConfluenceTool {
    fn name(&self) -> &str {
        "confluence"
    }

    fn description(&self) -> &str {
        "Search Confluence pages. Input should be a CQL query or search term."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        let url = format!("{}/rest/api/content/search?cql=text~%22{}%22&limit=5", self.base_url, urlencode(query));
        let resp = self.client.get(&url)
            .basic_auth(&self.email, Some(&self.api_token))
            .send().await
            .map_err(|e| ChainError::ToolError(format!("Confluence API error: {}", e)))?;
        let result: serde_json::Value = resp.json().await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
        let mut output = Vec::new();
        if let Some(results) = result["results"].as_array() {
            for page in results.iter().take(5) {
                let title = page["title"].as_str().unwrap_or("");
                let id = page["id"].as_str().unwrap_or("");
                let link = format!("{}/pages/viewpage.action?pageId={}", self.base_url, id);
                output.push(format!("{} - {}", title, link));
            }
        }
        if output.is_empty() {
            Ok("No pages found".into())
        } else {
            Ok(output.join("\n"))
        }
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
