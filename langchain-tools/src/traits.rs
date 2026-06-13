//! Core [`BaseTool`] trait for defining and invoking tools.

use async_trait::async_trait;
use langchain_core::errors::Result;

/// Convenience alias for a tool's return type.
pub type ToolResult = Result<String>;

/// Trait for LangChain tools that can be named, described, and invoked with a
/// string input.
#[async_trait]
pub trait BaseTool: Send + Sync {
    /// Returns the tool's name.
    fn name(&self) -> &str;
    /// Returns a description of what the tool does.
    fn description(&self) -> &str;
    /// Invokes the tool with a string input and returns a string result.
    async fn invoke(&self, input: &str) -> ToolResult;
    /// Converts this tool into a [`ToolDefinition`].
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
        }
    }
}

/// A descriptor for a tool, suitable for use in LLM function/tool definitions.
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    /// The tool's name.
    pub name: String,
    /// A description of what the tool does.
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoTool;

    #[async_trait]
    impl BaseTool for EchoTool {
        fn name(&self) -> &str { "echo" }
        fn description(&self) -> &str { "Echoes input" }
        async fn invoke(&self, input: &str) -> ToolResult {
            Ok(input.to_string())
        }
    }

    #[tokio::test]
    async fn test_tool_name() {
        let tool = EchoTool;
        assert_eq!(tool.name(), "echo");
    }

    #[tokio::test]
    async fn test_tool_description() {
        let tool = EchoTool;
        assert_eq!(tool.description(), "Echoes input");
    }

    #[tokio::test]
    async fn test_tool_invoke() {
        let tool = EchoTool;
        let result = tool.invoke("hello").await.unwrap();
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_tool_invoke_empty() {
        let tool = EchoTool;
        let result = tool.invoke("").await.unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_tool_definition() {
        let tool = EchoTool;
        let def = tool.to_definition();
        assert_eq!(def.name, "echo");
        assert_eq!(def.description, "Echoes input");
    }

    #[test]
    fn test_tool_definition_debug() {
        let def = ToolDefinition {
            name: "t".into(),
            description: "d".into(),
        };
        let s = format!("{:?}", def);
        assert!(s.contains("t"));
    }

    #[test]
    fn test_tool_trait_object() {
        let tool: &dyn BaseTool = &EchoTool;
        assert_eq!(tool.name(), "echo");
    }

    #[tokio::test]
    async fn test_tool_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<EchoTool>();
        assert_sync::<EchoTool>();
    }
}
