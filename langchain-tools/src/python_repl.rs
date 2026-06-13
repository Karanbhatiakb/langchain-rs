//! Python REPL execution tool.
//!
//! The [`PythonREPL`] tool runs Python code and returns its output. It
//! invokes a Python interpreter (default `python3`) with the provided
//! code as the `-c` argument.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};

/// Tool that executes Python code and returns the printed output.
///
/// # Input format
///
/// ```text
/// <Python code>
/// ```
///
/// # Stub
///
/// This is a stub implementation. Production use should replace the body of
/// [`invoke`](PythonREPL::invoke) with a real Python process spawn.
#[derive(Debug, Clone)]
pub struct PythonREPL;

impl PythonREPL {
    /// Creates a new [`PythonREPL`] tool.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonREPL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for PythonREPL {
    fn name(&self) -> &str {
        "python_repl"
    }

    fn description(&self) -> &str {
        "Executes Python code and returns its output. \
         Input should be valid Python code. Uses python3 interpreter."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from python_repl (stub)".to_string())
    }
}
