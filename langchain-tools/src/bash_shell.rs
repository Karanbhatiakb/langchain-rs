//! Bash shell execution tool.
//!
//! The [`BashShell`] tool runs shell commands and returns their standard
//! output. This is a lightweight alternative to [`ShellTool`] for
//! bash-specific command execution.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};

/// Tool that executes a shell command via `sh -c` and returns the output.
///
/// # Input format
///
/// ```text
/// <shell command>
/// ```
///
/// # Stub
///
/// This is a stub implementation. Production use should replace the body of
/// [`invoke`](BashShell::invoke) with a real process spawn.
#[derive(Debug, Clone)]
pub struct BashShell;

impl BashShell {
    /// Creates a new [`BashShell`] tool.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BashShell {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BashShell {
    fn name(&self) -> &str {
        "bash_shell"
    }

    fn description(&self) -> &str {
        "Executes a bash shell command and returns its output. \
         Input should be a valid shell command string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from bash_shell (stub)".to_string())
    }
}
