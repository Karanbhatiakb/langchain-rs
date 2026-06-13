//! Office 365 toolkit for interacting with Microsoft 365 services.

use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

/// Tool that sends emails via Office 365.
#[derive(Debug)]
pub struct Office365SendEmailTool;

#[async_trait]
impl BaseTool for Office365SendEmailTool {
    fn name(&self) -> &str {
        "office365_send_email"
    }

    fn description(&self) -> &str {
        "Sends an email via Office 365"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from office365_send_email (stub)".to_string())
    }
}

/// Tool that lists emails from Office 365.
#[derive(Debug)]
pub struct Office365ListEmailsTool;

#[async_trait]
impl BaseTool for Office365ListEmailsTool {
    fn name(&self) -> &str {
        "office365_list_emails"
    }

    fn description(&self) -> &str {
        "Lists recent emails from Office 365"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from office365_list_emails (stub)".to_string())
    }
}

/// Tool that creates calendar events in Office 365.
#[derive(Debug)]
pub struct Office365CreateEventTool;

#[async_trait]
impl BaseTool for Office365CreateEventTool {
    fn name(&self) -> &str {
        "office365_create_event"
    }

    fn description(&self) -> &str {
        "Creates a calendar event in Office 365"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from office365_create_event (stub)".to_string())
    }
}

/// A toolkit for interacting with Microsoft Office 365.
///
/// Provides tools for email, calendar, and contacts management.
#[derive(Debug)]
pub struct Office365Toolkit;

impl Office365Toolkit {
    /// Creates a new [`Office365Toolkit`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for Office365Toolkit {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseToolkit for Office365Toolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(Office365SendEmailTool) as Arc<dyn BaseTool>,
            Arc::new(Office365ListEmailsTool),
            Arc::new(Office365CreateEventTool),
        ]
    }

    fn name(&self) -> &str {
        "office365"
    }
}
