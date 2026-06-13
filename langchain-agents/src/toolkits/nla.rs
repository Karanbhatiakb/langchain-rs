//! Natural Language API (NLA) toolkit.
//!
//! The NLA toolkit wraps APIs that accept natural-language descriptions
//! of the desired action and translate them into structured API calls.

use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

/// Tool that executes a natural-language API action.
#[derive(Debug)]
pub struct NLAExecuteTool;

#[async_trait]
impl BaseTool for NLAExecuteTool {
    fn name(&self) -> &str {
        "nla_execute"
    }

    fn description(&self) -> &str {
        "Executes an API action described in natural language"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from nla_execute (stub)".to_string())
    }
}

/// Tool that lists available NLA actions.
#[derive(Debug)]
pub struct NLAActionsTool;

#[async_trait]
impl BaseTool for NLAActionsTool {
    fn name(&self) -> &str {
        "nla_actions"
    }

    fn description(&self) -> &str {
        "Lists available Natural Language API actions"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from nla_actions (stub)".to_string())
    }
}

/// A toolkit for Natural Language API (NLA) integrations.
///
/// Provides tools that translate natural-language descriptions into
/// structured API calls.
#[derive(Debug)]
pub struct NLAToolkit;

impl NLAToolkit {
    /// Creates a new [`NLAToolkit`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for NLAToolkit {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseToolkit for NLAToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(NLAExecuteTool) as Arc<dyn BaseTool>,
            Arc::new(NLAActionsTool),
        ]
    }

    fn name(&self) -> &str {
        "nla"
    }
}
