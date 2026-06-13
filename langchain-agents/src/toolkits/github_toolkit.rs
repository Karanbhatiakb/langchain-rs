use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct GitHubSearchTool;

#[async_trait]
impl BaseTool for GitHubSearchTool {
    fn name(&self) -> &str {
        "github_search"
    }

    fn description(&self) -> &str {
        "Searches GitHub"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from github_search (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct GitHubCreateIssueTool;

#[async_trait]
impl BaseTool for GitHubCreateIssueTool {
    fn name(&self) -> &str {
        "github_create_issue"
    }

    fn description(&self) -> &str {
        "Creates a GitHub issue"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from github_create_issue (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct GitHubReadFileTool;

#[async_trait]
impl BaseTool for GitHubReadFileTool {
    fn name(&self) -> &str {
        "github_read_file"
    }

    fn description(&self) -> &str {
        "Reads a file from GitHub"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from github_read_file (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct GitHubToolkit;

impl GitHubToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for GitHubToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(GitHubSearchTool) as Arc<dyn BaseTool>,
            Arc::new(GitHubCreateIssueTool) as Arc<dyn BaseTool>,
            Arc::new(GitHubReadFileTool),
        ]
    }

    fn name(&self) -> &str {
        "github"
    }
}
