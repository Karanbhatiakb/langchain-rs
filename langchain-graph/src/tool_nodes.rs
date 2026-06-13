//! Tool node that wraps a tool and integrates it into a graph.

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use langchain_core::messages::MessageType;
use langchain_tools::traits::BaseTool;
use std::sync::Arc;

use crate::nodes::Node;
use crate::state::AgentState;
use crate::state::StateSchema;

pub struct ToolNode {
    name: String,
    tool: Arc<dyn BaseTool>,
}

impl ToolNode {
    pub fn new(name: impl Into<String>, tool: Arc<dyn BaseTool>) -> Self {
        Self {
            name: name.into(),
            tool,
        }
    }

    pub fn tool_name(&self) -> &str {
        self.tool.name()
    }

    pub fn tool_description(&self) -> &str {
        self.tool.description()
    }
}

#[async_trait]
impl Node<AgentState> for ToolNode {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, state: AgentState) -> Result<AgentState> {
        let tool_input = state
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let result = self.tool.invoke(&tool_input).await?;

        let ai_msg = BaseMessage::new(result, MessageType::AI);

        let update = crate::state::AgentStateUpdate::new().with_messages(vec![ai_msg]);

        let mut new_state = state;
        new_state.merge(update);

        Ok(new_state)
    }
}

pub struct ToolNodeConfig {
    pub name: String,
    pub tool: Arc<dyn BaseTool>,
}

impl ToolNodeConfig {
    pub fn new(name: impl Into<String>, tool: Arc<dyn BaseTool>) -> Self {
        Self {
            name: name.into(),
            tool,
        }
    }
}
