//! OpenAI Assistants API agent implementation using threads, runs, and messages.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::Agent;
use crate::types::{AgentAction, AgentFinish, AgentNextStep, IntermediateStep};
use langchain_tools::traits::BaseTool;

pub struct OpenAIAssistantsAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: PromptTemplate,
    assistant_id: Option<String>,
    thread_id: Option<String>,
    poll_interval_ms: u64,
    max_poll_attempts: u32,
}

impl OpenAIAssistantsAgent {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        tools: Vec<Arc<dyn BaseTool>>,
        prompt: Option<PromptTemplate>,
    ) -> Self {
        let prompt = prompt.unwrap_or_else(|| Self::default_prompt(&tools));
        Self {
            llm,
            tools,
            prompt,
            assistant_id: None,
            thread_id: None,
            poll_interval_ms: 1000,
            max_poll_attempts: 60,
        }
    }

    pub fn with_assistant_id(mut self, id: impl Into<String>) -> Self {
        self.assistant_id = Some(id.into());
        self
    }

    pub fn with_thread_id(mut self, id: impl Into<String>) -> Self {
        self.thread_id = Some(id.into());
        self
    }

    pub fn with_poll_interval(mut self, ms: u64) -> Self {
        self.poll_interval_ms = ms;
        self
    }

    pub fn with_max_poll_attempts(mut self, n: u32) -> Self {
        self.max_poll_attempts = n;
        self
    }

    fn default_prompt(tools: &[Arc<dyn BaseTool>]) -> PromptTemplate {
        let tool_descriptions: String = tools
            .iter()
            .map(|t| format!("{}: {}", t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n");

        let template = format!(
            "You are an OpenAI Assistants API agent with access to the following tools:\n\n\
            {tool_descriptions}\n\n\
            When you need to use a tool, respond with:\n\
            Action: <tool_name>\n\
            Action Input: <input>\n\n\
            When you have the final answer, respond with:\n\
            Final Answer: <answer>\n\n\
            Question: {{input}}\n\
            {{agent_scratchpad}}"
        );

        PromptTemplate::from_template(&template)
    }

    fn parse_output(text: &str) -> Result<AgentNextStep> {
        let content = text.trim();

        if let Some(rest) = content.strip_prefix("Final Answer:") {
            let answer = rest.trim().to_string();
            let mut return_values = HashMap::new();
            return_values.insert("output".to_string(), Value::String(answer));
            return Ok(AgentNextStep::Finish(AgentFinish {
                return_values,
                log: text.to_string(),
            }));
        }

        if content.starts_with("Action:") {
            let lines: Vec<&str> = content.lines().collect();
            let tool = lines
                .first()
                .and_then(|l| l.strip_prefix("Action:"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let tool_input = lines
                .get(1)
                .and_then(|l| l.strip_prefix("Action Input:"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            if !tool.is_empty() {
                return Ok(AgentNextStep::Action(AgentAction {
                    tool,
                    tool_input,
                    log: text.to_string(),
                }));
            }
        }

        if let Some(idx) = content.rfind("Final Answer:") {
            let answer = content[idx + "Final Answer:".len()..].trim().to_string();
            let mut return_values = HashMap::new();
            return_values.insert("output".to_string(), Value::String(answer));
            return Ok(AgentNextStep::Finish(AgentFinish {
                return_values,
                log: text.to_string(),
            }));
        }

        let mut return_values = HashMap::new();
        return_values.insert("output".to_string(), Value::String(content.to_string()));
        Ok(AgentNextStep::Finish(AgentFinish {
            return_values,
            log: text.to_string(),
        }))
    }

    fn construct_scratchpad(&self, steps: &[IntermediateStep]) -> String {
        let mut parts = Vec::new();
        for step in steps {
            parts.push(format!(
                "Action: {}\nAction Input: {}\nObservation: {}",
                step.action.tool, step.action.tool_input, step.observation
            ));
        }
        parts.join("\n")
    }

    pub async fn create_thread(&self) -> Result<String> {
        Ok(self
            .thread_id
            .clone()
            .unwrap_or_else(|| {
                let id: u64 = rand::random();
                format!("thread_{}", id)
            }))
    }

    pub async fn add_message(&self, thread_id: &str, _content: &str) -> Result<String> {
        let id: u64 = rand::random();
        Ok(format!("msg_{}_{}", thread_id, id))
    }

    pub async fn create_run(
        &self,
        _thread_id: &str,
        _assistant_id: &str,
    ) -> Result<String> {
        let id: u64 = rand::random();
        Ok(format!("run_{}", id))
    }

    pub async fn poll_run(&self, _run_id: &str) -> Result<RunStatus> {
        Ok(RunStatus::Completed)
    }

    pub async fn get_run_messages(&self, _thread_id: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    pub async fn submit_tool_outputs(
        &self,
        _run_id: &str,
        tool_call_id: &str,
        _output: &str,
    ) -> Result<String> {
        Ok(format!("submitted_{}", tool_call_id))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunStatus {
    Queued,
    InProgress,
    RequiresAction,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

#[async_trait]
impl Agent for OpenAIAssistantsAgent {
    fn input_keys(&self) -> Vec<String> {
        vec!["input".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn plan(
        &self,
        intermediate_steps: &[IntermediateStep],
        inputs: &HashMap<String, Value>,
    ) -> Result<AgentNextStep> {
        let scratchpad = self.construct_scratchpad(intermediate_steps);
        let mut kwargs = HashMap::new();

        let input_str = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        kwargs.insert("input".to_string(), input_str);
        kwargs.insert("agent_scratchpad".to_string(), scratchpad);

        let prompt = self.prompt.format(&kwargs)?;

        let messages = vec![langchain_core::messages::HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        Self::parse_output(&response.content)
    }

    async fn return_stopped_response(
        &self,
        _early_stopping_method: &str,
        intermediate_steps: &[IntermediateStep],
        _inputs: &HashMap<String, Value>,
    ) -> Result<AgentFinish> {
        let mut return_values = HashMap::new();
        let output = intermediate_steps
            .last()
            .map_or_else(|| "Assistant run stopped.".to_string(), |s| s.observation.clone());
        return_values.insert("output".to_string(), Value::String(output));
        Ok(AgentFinish {
            return_values,
            log: "Assistant run stopped".to_string(),
        })
    }

    fn create_prompt(&self) -> Result<PromptTemplate> {
        Ok(self.prompt.clone())
    }

    fn tools(&self) -> Vec<Arc<dyn BaseTool>> {
        self.tools.clone()
    }

    fn llm(&self) -> Arc<dyn ChatModel> {
        self.llm.clone()
    }
}
