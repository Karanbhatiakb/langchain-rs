//! OpenAI function-calling agent implementation.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{BaseMessage, HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::{ChatModel, FunctionDefinition};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::Agent;
use crate::types::{AgentAction, AgentFinish, AgentNextStep, IntermediateStep};
use langchain_tools::traits::BaseTool;

pub struct OpenAIFunctionsAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: PromptTemplate,
    function_definitions: Vec<FunctionDefinition>,
}

impl OpenAIFunctionsAgent {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        tools: Vec<Arc<dyn BaseTool>>,
        prompt: Option<PromptTemplate>,
    ) -> Self {
        let function_definitions = Self::convert_tools_to_functions(&tools);
        let prompt = prompt.unwrap_or_else(|| Self::default_prompt());
        Self {
            llm,
            tools,
            prompt,
            function_definitions,
        }
    }

    fn convert_tools_to_functions(tools: &[Arc<dyn BaseTool>]) -> Vec<FunctionDefinition> {
        tools
            .iter()
            .map(|t| FunctionDefinition {
            name: t.name().to_string(),
            description: t.description().to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": format!("Input for the {} tool", t.name())
                        }
                    },
                    "required": ["input"]
                }),
            })
            .collect()
    }

    fn default_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            "You are a helpful AI assistant. Use the provided functions to answer the question.\n\nQuestion: {input}\n\n{agent_scratchpad}",
        )
    }

    fn parse_response(message: &BaseMessage) -> Result<AgentNextStep> {
        if let Some(function_call) = message.additional_kwargs.get("function_call") {
            let name = function_call
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let arguments = function_call
                .get("arguments")
                .and_then(|v| v.as_str())
                .unwrap_or("{}")
                .to_string();

            let tool_input = serde_json::from_str::<Value>(&arguments)
                .ok()
                .and_then(|v| v.get("input").and_then(|i| i.as_str().map(|s| s.to_string())))
                .unwrap_or(arguments);

            return Ok(AgentNextStep::Action(AgentAction {
                tool: name,
                tool_input,
                log: message.content.clone(),
            }));
        }

        if !message.content.is_empty() {
            let mut return_values = HashMap::new();
            return_values.insert("output".to_string(), Value::String(message.content.clone()));
            return Ok(AgentNextStep::Finish(AgentFinish {
                return_values,
                log: message.content.clone(),
            }));
        }

        Err(ChainError::ParserError(
            "No function call or content in response".to_string(),
        ))
    }
}

#[async_trait]
impl Agent for OpenAIFunctionsAgent {
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

        let messages: Vec<BaseMessage> = vec![
            SystemMessage::new("You are a helpful AI assistant. Respond using function calls.").into(),
            HumanMessage::new(&formatted).into(),
        ];

        let response = self
            .llm
            .predict_messages(&messages, Some(&self.function_definitions), None)
            .await?;

        Self::parse_response(&response)
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

impl OpenAIFunctionsAgent {
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
