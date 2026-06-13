use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct FileReadTool;

#[async_trait]
impl BaseTool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Reads the contents of a file"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from file_read (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct FileWriteTool;

#[async_trait]
impl BaseTool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Writes content to a file"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from file_write (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct FileListTool;

#[async_trait]
impl BaseTool for FileListTool {
    fn name(&self) -> &str {
        "file_list"
    }

    fn description(&self) -> &str {
        "Lists files in a directory"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from file_list (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct DirectoryListTool;

#[async_trait]
impl BaseTool for DirectoryListTool {
    fn name(&self) -> &str {
        "directory_list"
    }

    fn description(&self) -> &str {
        "Lists directories"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from directory_list (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct FileManagementToolkit;

impl FileManagementToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for FileManagementToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(FileReadTool) as Arc<dyn BaseTool>,
            Arc::new(FileWriteTool) as Arc<dyn BaseTool>,
            Arc::new(FileListTool) as Arc<dyn BaseTool>,
            Arc::new(DirectoryListTool),
        ]
    }

    fn name(&self) -> &str {
        "file_management"
    }
}
