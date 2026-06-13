//! GitHub API tool.

use async_trait::async_trait;
use serde_json::{json, Value};

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct GitHubTool {
    token: String,
    client: reqwest::Client,
}

impl GitHubTool {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );
        headers.insert(
            "Accept",
            reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        headers
    }

    async fn api_get(&self, path: &str) -> Result<Value, ChainError> {
        let url = format!("https://api.github.com{}", path);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .header("User-Agent", "langchain-rs")
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("GitHub API error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::ToolError(format!(
                "GitHub API returned {}",
                resp.status()
            )));
        }

        resp.json::<Value>()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to parse response: {}", e)))
    }

    async fn api_post(&self, path: &str, body: &Value) -> Result<Value, ChainError> {
        let url = format!("https://api.github.com{}", path);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .header("User-Agent", "langchain-rs")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("GitHub API error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::ToolError(format!(
                "GitHub API returned {}",
                resp.status()
            )));
        }

        resp.json::<Value>()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to parse response: {}", e)))
    }
}

#[async_trait]
impl BaseTool for GitHubTool {
    fn name(&self) -> &str {
        "github"
    }

    fn description(&self) -> &str {
        "GitHub API tool. Supports: get_repo <owner/repo>, list_issues <owner/repo>, create_issue <owner/repo>\\n<title>\\n<body>, search_code <query>, get_file_contents <owner/repo>\\n<path>, list_pull_requests <owner/repo>, create_pull_request <owner/repo>\\n<title>\\n<head>\\n<base>\\n<body>. Requires GITHUB_TOKEN env var."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if let Some(cmd) = input.strip_prefix("get_repo ") {
            let result = self.api_get(&format!("/repos/{}", cmd.trim())).await?;
            return Ok(format!(
                "Repository: {}\nDescription: {}\nStars: {}\nForks: {}\nLanguage: {}",
                result["full_name"].as_str().unwrap_or(""),
                result["description"].as_str().unwrap_or(""),
                result["stargazers_count"].as_i64().unwrap_or(0),
                result["forks_count"].as_i64().unwrap_or(0),
                result["language"].as_str().unwrap_or(""),
            ));
        }

        if let Some(cmd) = input.strip_prefix("list_issues ") {
            let result = self.api_get(&format!("/repos/{}/issues", cmd.trim())).await?;
            let issues = result.as_array().ok_or_else(|| ChainError::ToolError("Expected array".into()))?;
            let mut output = Vec::new();
            for issue in issues {
                output.push(format!(
                    "#{} - {} ({})",
                    issue["number"].as_i64().unwrap_or(0),
                    issue["title"].as_str().unwrap_or(""),
                    issue["state"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("create_issue ") {
            let parts: Vec<&str> = cmd.splitn(3, '\n').collect();
            if parts.len() < 3 {
                return Err(ChainError::ToolError("Usage: create_issue <owner/repo>\\n<title>\\n<body>".into()));
            }
            let repo = parts[0].trim();
            let title = parts[1].trim();
            let body = parts[2].trim();
            let result = self
                .api_post(
                    &format!("/repos/{}/issues", repo),
                    &json!({"title": title, "body": body}),
                )
                .await?;
            return Ok(format!(
                "Created issue #{}: {}",
                result["number"].as_i64().unwrap_or(0),
                result["html_url"].as_str().unwrap_or(""),
            ));
        }

        if let Some(cmd) = input.strip_prefix("search_code ") {
            let query = cmd.trim();
            let result = self.api_get(&format!("/search/code?q={}", urlencode(query))).await?;
            let items = result["items"].as_array().ok_or_else(|| ChainError::ToolError("Expected items array".into()))?;
            let mut output = Vec::new();
            for item in items {
                output.push(format!(
                    "{}: {}",
                    item["repository"]["full_name"].as_str().unwrap_or(""),
                    item["path"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("get_file_contents ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("Usage: get_file_contents <owner/repo>\\n<path>".into()));
            }
            let repo = parts[0].trim();
            let path = parts[1].trim();
            let result = self
                .api_get(&format!("/repos/{}/contents/{}", repo, path.trim_start_matches('/')))
                .await?;
            let content = result["content"].as_str().unwrap_or("");
            let decoded = base64_decode(content);
            return Ok(decoded);
        }

        if let Some(cmd) = input.strip_prefix("list_pull_requests ") {
            let result = self.api_get(&format!("/repos/{}/pulls", cmd.trim())).await?;
            let prs = result.as_array().ok_or_else(|| ChainError::ToolError("Expected array".into()))?;
            let mut output = Vec::new();
            for pr in prs {
                output.push(format!(
                    "#{} - {} ({})",
                    pr["number"].as_i64().unwrap_or(0),
                    pr["title"].as_str().unwrap_or(""),
                    pr["state"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("create_pull_request ") {
            let parts: Vec<&str> = cmd.splitn(4, '\n').collect();
            if parts.len() < 4 {
                return Err(ChainError::ToolError("Usage: create_pull_request <owner/repo>\\n<title>\\n<head>\\n<base>\\n<body>".into()));
            }
            let repo = parts[0].trim();
            let title = parts[1].trim();
            let head = parts[2].trim();
            let base = parts[3].trim();
            let body = if parts.len() > 4 { parts[4].trim() } else { "" };
            let result = self
                .api_post(
                    &format!("/repos/{}/pulls", repo),
                    &json!({"title": title, "head": head, "base": base, "body": body}),
                )
                .await?;
            return Ok(format!(
                "Created PR #{}: {}",
                result["number"].as_i64().unwrap_or(0),
                result["html_url"].as_str().unwrap_or(""),
            ));
        }

        Err(ChainError::ToolError(format!(
            "Unknown GitHub command. Supported: get_repo, list_issues, create_issue, search_code, get_file_contents, list_pull_requests, create_pull_request"
        )))
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' | '/' => c.to_string(),
            ' ' => '+'.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn base64_decode(input: &str) -> String {
    use base64::Engine;
    let input = input.replace('\n', "");
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&input) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    }
}
