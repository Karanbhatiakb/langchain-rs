//! Tool-calling agent implementation.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{BaseMessage, HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::{ChatModel, ToolDefinition};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::Agent;
use crate::types::{AgentAction, AgentFinish, AgentNextStep, IntermediateStep};
use langchain_tools::traits::BaseTool;

pub struct ToolCallingAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: PromptTemplate,
    tool_definitions: Vec<ToolDefinition>,
}

impl ToolCallingAgent {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        tools: Vec<Arc<dyn BaseTool>>,
        prompt: Option<PromptTemplate>,
    ) -> Self {
        let tool_definitions: Vec<ToolDefinition> = tools
            .iter()
            .map(|t| ToolDefinition {
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
            .collect();

        let prompt = prompt.unwrap_or_else(|| Self::default_prompt());
        Self {
            llm,
            tools,
            prompt,
            tool_definitions,
        }
    }

    fn default_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            "You are a helpful AI assistant with access to tools.\n\nQuestion: {input}\n\n{agent_scratchpad}",
        )
    }

    fn parse_response(message: &BaseMessage) -> Result<Vec<AgentNextStep>> {
        let tool_calls = message.additional_kwargs.get("tool_calls");
        if let Some(calls) = tool_calls.and_then(|v| v.as_array()) {
            if !calls.is_empty() {
                let steps: Vec<AgentNextStep> = calls
                    .iter()
                    .filter_map(|call| {
                        let name = call
                            .get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|v| v.as_str())?;
                        let args = call
                            .get("function")
                            .and_then(|f| f.get("arguments"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("{}");

                        let tool_input = serde_json::from_str::<Value>(args)
                            .ok()
                            .and_then(|v| {
                                v.get("input")
                                    .and_then(|i| i.as_str().map(|s| s.to_string()))
                            })
                            .unwrap_or_else(|| args.to_string());

                        Some(AgentNextStep::Action(AgentAction {
                            tool: name.to_string(),
                            tool_input,
                            log: message.content.clone(),
                        }))
                    })
                    .collect();

                if !steps.is_empty() {
                    return Ok(steps);
                }
            }
        }

        if !message.content.is_empty() {
            let mut return_values = HashMap::new();
            return_values.insert("output".to_string(), Value::String(message.content.clone()));
            return Ok(vec![AgentNextStep::Finish(AgentFinish {
                return_values,
                log: message.content.clone(),
            })]);
        }

        Err(ChainError::ParserError(
            "No tool calls or content in response".to_string(),
        ))
    }
}

#[async_trait]
impl Agent for ToolCallingAgent {
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

        let llm_with_tools = self.llm.bind_tools(self.tool_definitions.clone());

        let messages: Vec<BaseMessage> = vec![
            SystemMessage::new("You are a helpful AI assistant with access to tools.").into(),
            HumanMessage::new(&formatted).into(),
        ];

        let response = llm_with_tools
            .predict_messages(&messages, None, None)
            .await?;

        let steps = Self::parse_response(&response)?;
        Ok(steps.into_iter().next().unwrap_or_else(|| {
            AgentNextStep::Finish(AgentFinish {
                return_values: {
                    let mut m = HashMap::new();
                    m.insert("output".to_string(), Value::String(response.content.clone()));
                    m
                },
                log: response.content,
            })
        }))
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

impl ToolCallingAgent {
    fn construct_scratchpad(&self, steps: &[IntermediateStep]) -> String {
        let mut parts = Vec::new();
        for step in steps {
            parts.push(format!(
                "Called tool: {} with input: {}\nObservation: {}",
                step.action.tool, step.action.tool_input, step.observation
            ));
        }
        parts.join("\n")
    }
}
