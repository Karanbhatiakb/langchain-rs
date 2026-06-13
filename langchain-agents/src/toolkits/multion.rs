//! MultiOn toolkit for AI-powered browser control.

use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

/// Tool that browses the web using MultiOn AI.
#[derive(Debug)]
pub struct MultiOnBrowseTool;

#[async_trait]
impl BaseTool for MultiOnBrowseTool {
    fn name(&self) -> &str {
        "multion_browse"
    }

    fn description(&self) -> &str {
        "Browses the web using MultiOn AI to accomplish a task"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from multion_browse (stub)".to_string())
    }
}

/// Tool that extracts structured data from web pages via MultiOn.
#[derive(Debug)]
pub struct MultiOnExtractTool;

#[async_trait]
impl BaseTool for MultiOnExtractTool {
    fn name(&self) -> &str {
        "multion_extract"
    }

    fn description(&self) -> &str {
        "Extracts structured data from web pages using MultiOn AI"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from multion_extract (stub)".to_string())
    }
}

/// A toolkit for controlling browsers with MultiOn AI.
///
/// Provides tools for autonomous web browsing and data extraction.
#[derive(Debug)]
pub struct MultiOnToolkit;

impl MultiOnToolkit {
    /// Creates a new [`MultiOnToolkit`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for MultiOnToolkit {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseToolkit for MultiOnToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(MultiOnBrowseTool) as Arc<dyn BaseTool>,
            Arc::new(MultiOnExtractTool),
        ]
    }

    fn name(&self) -> &str {
        "multion"
    }
}
