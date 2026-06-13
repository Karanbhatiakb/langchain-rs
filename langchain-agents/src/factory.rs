//! Agent factory — creates agent from config.

use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use langchain_tools::traits::BaseTool;
use std::sync::Arc;

use crate::executor::AgentExecutor;

#[cfg(feature = "react")]
pub fn create_react_agent(
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: Option<PromptTemplate>,
) -> AgentExecutor {
    let agent = Arc::new(crate::react::ReActAgent::new(llm, tools.clone(), prompt));
    AgentExecutor::new(agent, tools)
}

#[cfg(feature = "tool_calling")]
pub fn create_tool_calling_agent(
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: Option<PromptTemplate>,
) -> AgentExecutor {
    let agent = Arc::new(crate::tool_calling::ToolCallingAgent::new(
        llm, tools.clone(), prompt,
    ));
    AgentExecutor::new(agent, tools)
}

#[cfg(feature = "openai_functions")]
pub fn create_openai_functions_agent(
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: Option<PromptTemplate>,
) -> AgentExecutor {
    let agent = Arc::new(crate::openai_functions::OpenAIFunctionsAgent::new(
        llm, tools.clone(), prompt,
    ));
    AgentExecutor::new(agent, tools)
}

#[cfg(feature = "json")]
pub fn create_json_agent(
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: Option<PromptTemplate>,
) -> AgentExecutor {
    let agent = Arc::new(crate::json_agent::JSONAgent::new(llm, tools.clone(), prompt));
    AgentExecutor::new(agent, tools)
}

#[cfg(feature = "structured_chat")]
pub fn create_structured_chat_agent(
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: Option<PromptTemplate>,
) -> AgentExecutor {
    let agent = Arc::new(crate::structured_chat::StructuredChatAgent::new(
        llm, tools.clone(), prompt,
    ));
    AgentExecutor::new(agent, tools)
}

#[cfg(feature = "xml")]
pub fn create_xml_agent(
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: Option<PromptTemplate>,
) -> AgentExecutor {
    let agent = Arc::new(crate::xml::XMLAgent::new(llm, tools.clone(), prompt));
    AgentExecutor::new(agent, tools)
}
