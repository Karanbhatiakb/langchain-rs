//! JSON agent implementation.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::Agent;
use crate::types::{AgentAction, AgentFinish, AgentNextStep, IntermediateStep};
use langchain_tools::traits::BaseTool;

pub struct JSONAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: PromptTemplate,
}

impl JSONAgent {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        tools: Vec<Arc<dyn BaseTool>>,
        prompt: Option<PromptTemplate>,
    ) -> Self {
        let prompt = prompt.unwrap_or_else(|| Self::default_prompt(&tools));
        Self { llm, tools, prompt }
    }

    fn default_prompt(tools: &[Arc<dyn BaseTool>]) -> PromptTemplate {
        let tool_descriptions: String = tools
            .iter()
            .map(|t| format!("\"{}\": {}", t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n");

        let tool_names: Vec<String> = tools.iter().map(|t| t.name().to_string()).collect();
        let tool_names_str = tool_names.join(", ");

        let template = format!(
            "You are a helpful AI assistant. You have access to the following tools:\n\n\
            {tool_descriptions}\n\n\
            Respond with JSON in this format:\n\n\
            To use a tool:\n\
            {{{{ \"tool\": \"<tool_name>\", \"tool_input\": \"<input>\" }}}}\n\n\
            To give a final answer:\n\
            {{{{ \"output\": \"your final answer\" }}}}\n\n\
            Available tools: [{tool_names_str}]\n\n\
            Question: {{{{input}}}}\n\n\
            {{{{agent_scratchpad}}}}"
        );

        PromptTemplate::from_template(&template)
    }

    fn parse_output(text: &str) -> Result<AgentNextStep> {
        let trimmed = text.trim();

        let json_start = trimmed
            .find('{')
            .ok_or_else(|| ChainError::ParserError("No JSON found".to_string()))?;
        let json_str = &trimmed[json_start..];

        let json_end = json_str
            .rfind('}')
            .ok_or_else(|| ChainError::ParserError("Unclosed JSON".to_string()))?;
        let json_str = &json_str[..=json_end];

        let parsed: Value =
            serde_json::from_str(json_str).map_err(|e| ChainError::ParserError(e.to_string()))?;

        if let Some(output) = parsed.get("output").and_then(|v| v.as_str()) {
            let mut return_values = HashMap::new();
            return_values.insert("output".to_string(), Value::String(output.to_string()));
            return Ok(AgentNextStep::Finish(AgentFinish {
                return_values,
                log: text.to_string(),
            }));
        }

        if let (Some(tool), Some(input)) = (
            parsed.get("tool").and_then(|v| v.as_str()),
            parsed.get("tool_input").and_then(|v| v.as_str()),
        ) {
            return Ok(AgentNextStep::Action(AgentAction {
                tool: tool.to_string(),
                tool_input: input.to_string(),
                log: text.to_string(),
            }));
        }

        Err(ChainError::ParserError(format!(
            "Could not parse JSON agent output: {}",
            text
        )))
    }
}

#[async_trait]
impl Agent for JSONAgent {
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
            SystemMessage::new("You are a helpful AI assistant. Respond in JSON format.").into(),
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

impl JSONAgent {
    fn construct_scratchpad(&self, steps: &[IntermediateStep]) -> String {
        let mut parts = Vec::new();
        for step in steps {
            parts.push(format!(
                "Observation: {}",
                step.observation
            ));
        }
        parts.join("\n")
    }
}
