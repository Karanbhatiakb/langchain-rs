use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct ZapierListTool;

#[async_trait]
impl BaseTool for ZapierListTool {
    fn name(&self) -> &str {
        "zapier_list"
    }

    fn description(&self) -> &str {
        "Lists available Zapier actions"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from zapier_list (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct ZapierRunTool;

#[async_trait]
impl BaseTool for ZapierRunTool {
    fn name(&self) -> &str {
        "zapier_run"
    }

    fn description(&self) -> &str {
        "Runs a Zapier action"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from zapier_run (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct ZapierToolkit;

impl ZapierToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for ZapierToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![Arc::new(ZapierListTool) as Arc<dyn BaseTool>, Arc::new(ZapierRunTool)]
    }

    fn name(&self) -> &str {
        "zapier"
    }
}
