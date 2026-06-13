//! File system interaction tool.

use async_trait::async_trait;
use tokio::fs;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct ReadFileTool;

#[async_trait]
impl BaseTool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Input should be a file path."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let path = input.trim();
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to read file '{}': {}", path, e)))?;
        Ok(content)
    }
}

pub struct WriteFileTool;

#[async_trait]
impl BaseTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Input should be the file path, followed by a newline, then the content to write."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return Err(ChainError::ToolError(
                "Input must contain file path and content separated by a newline".into(),
            ));
        }
        let path = parts[0].trim();
        let content = parts[1];
        fs::write(path, content)
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to write file '{}': {}", path, e)))?;
        Ok(format!("Successfully wrote to {}", path))
    }
}

pub struct ListDirectoryTool;

#[async_trait]
impl BaseTool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List files and directories in a directory. Input should be a directory path."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let path = input.trim();
        let mut entries = fs::read_dir(path)
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to list directory '{}': {}", path, e)))?;

        let mut items = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ChainError::ToolError(format!("Error reading directory entry: {}", e)))?
        {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry
                .file_type()
                .await
                .map(|t| t.is_dir())
                .unwrap_or(false);
            if is_dir {
                items.push(format!("{}/", name));
            } else {
                items.push(name);
            }
        }

        items.sort();
        Ok(items.join("\n"))
    }
}

pub struct CopyFileTool;

#[async_trait]
impl BaseTool for CopyFileTool {
    fn name(&self) -> &str {
        "copy_file"
    }

    fn description(&self) -> &str {
        "Copy a file from source to destination. Input should be the source path, followed by a newline, then the destination path."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return Err(ChainError::ToolError(
                "Input must contain source and destination paths separated by a newline".into(),
            ));
        }
        let src = parts[0].trim();
        let dst = parts[1].trim();
        fs::copy(src, dst)
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to copy '{}' to '{}': {}", src, dst, e)))?;
        Ok(format!("Successfully copied {} to {}", src, dst))
    }
}

pub struct MoveFileTool;

#[async_trait]
impl BaseTool for MoveFileTool {
    fn name(&self) -> &str {
        "move_file"
    }

    fn description(&self) -> &str {
        "Move or rename a file. Input should be the source path, followed by a newline, then the destination path."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return Err(ChainError::ToolError(
                "Input must contain source and destination paths separated by a newline".into(),
            ));
        }
        let src = parts[0].trim();
        let dst = parts[1].trim();
        fs::rename(src, dst)
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to move '{}' to '{}': {}", src, dst, e)))?;
        Ok(format!("Successfully moved {} to {}", src, dst))
    }
}

pub struct FileSearchTool;

#[async_trait]
impl BaseTool for FileSearchTool {
    fn name(&self) -> &str {
        "file_search"
    }

    fn description(&self) -> &str {
        "Search for files matching a pattern. Input should be a glob pattern (e.g., '**/*.rs')."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let pattern = input.trim();
        let entries = glob::glob(pattern)
            .map_err(|e| ChainError::ToolError(format!("Invalid glob pattern: {}", e)))?;

        let mut results = Vec::new();
        for entry in entries {
            match entry {
                Ok(path) => results.push(path.to_string_lossy().to_string()),
                Err(e) => results.push(format!("Error: {}", e)),
            }
        }

        results.sort();
        Ok(results.join("\n"))
    }
}

pub struct DeleteFileTool;

#[async_trait]
impl BaseTool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "Delete a file. Input should be a file path."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let path = input.trim();
        fs::remove_file(path)
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to delete '{}': {}", path, e)))?;
        Ok(format!("Successfully deleted {}", path))
    }
}
