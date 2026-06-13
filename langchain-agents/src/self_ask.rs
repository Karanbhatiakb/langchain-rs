//! Self-ask agent implementation.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::Agent;
use crate::types::{AgentAction, AgentFinish, AgentNextStep, IntermediateStep};
use langchain_tools::traits::BaseTool;

pub struct SelfAskWithSearchAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: PromptTemplate,
    #[allow(dead_code)]
    search_tool_name: String,
}

impl SelfAskWithSearchAgent {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        tools: Vec<Arc<dyn BaseTool>>,
        prompt: Option<PromptTemplate>,
        search_tool_name: Option<String>,
    ) -> Self {
        let prompt = prompt.unwrap_or_else(|| Self::default_prompt());
        let search_tool_name = search_tool_name.unwrap_or_else(|| "search".to_string());
        Self {
            llm,
            tools,
            prompt,
            search_tool_name,
        }
    }

    fn default_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            r#"Question: {{input}}

You are a helpful AI assistant. You have access to a search tool.
Think step by step. If you need more information, generate a follow-up question.

To use the search tool, respond with:
Follow up question: your question here
Intermediate answer: the result from search

When you have enough information, respond with:
Final answer: your final answer

{{agent_scratchpad}}"#,
        )
    }

    fn parse_output(text: &str) -> Result<AgentNextStep> {
        let final_re =
            Regex::new(r"(?s)Final answer:\s*(.*)")
                .map_err(|e| ChainError::ParserError(e.to_string()))?;
        if let Some(caps) = final_re.captures(text) {
            let answer = caps.get(1).unwrap().as_str().trim().to_string();
            let mut return_values = HashMap::new();
            return_values.insert("output".to_string(), Value::String(answer));
            return Ok(AgentNextStep::Finish(AgentFinish {
                return_values,
                log: text.to_string(),
            }));
        }

        let follow_up_re =
            Regex::new(r"(?s)Follow up question:\s*(.*?)(?:\n|$)")
                .map_err(|e| ChainError::ParserError(e.to_string()))?;
        if let Some(caps) = follow_up_re.captures(text) {
            let question = caps.get(1).unwrap().as_str().trim().to_string();
            return Ok(AgentNextStep::Action(AgentAction {
                tool: "search".to_string(),
                tool_input: question,
                log: text.to_string(),
            }));
        }

        Err(ChainError::ParserError(format!(
            "Could not parse SelfAsk output: {}",
            text
        )))
    }
}

#[async_trait]
impl Agent for SelfAskWithSearchAgent {
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

        let formatted = self.prompt.format(&kwargs)?;

        let messages = vec![
            SystemMessage::new("You are a helpful assistant that uses search to answer questions.").into(),
            HumanMessage::new(&formatted).into(),
        ];

        let response = self.llm.predict_messages(&messages, None, None).await?;

        Self::parse_output(&response.content)
    }

    async fn return_stopped_response(
        &self,
        _early_stopping_method: &str,
        _intermediate_steps: &[IntermediateStep],
        _inputs: &HashMap<String, Value>,
    ) -> Result<AgentFinish> {
        let mut return_values = HashMap::new();
        return_values.insert(
            "output".to_string(),
            Value::String("Agent stopped.".to_string()),
        );
        Ok(AgentFinish {
            return_values,
            log: "Agent stopped due to max iterations".to_string(),
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

impl SelfAskWithSearchAgent {
    fn construct_scratchpad(&self, steps: &[IntermediateStep]) -> String {
        let mut parts = Vec::new();
        for step in steps {
            parts.push(format!(
                "Follow up question: {}\nIntermediate answer: {}",
                step.action.tool_input, step.observation
            ));
        }
        parts.join("\n")
    }
}
