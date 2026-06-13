//! MRKL (Modular Reasoning, Knowledge and Language) agent implementation.

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

pub struct MRKLAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    prompt: PromptTemplate,
    max_tokens: Option<u32>,
}

impl MRKLAgent {
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
            max_tokens: None,
        }
    }

    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }

    fn default_prompt(tools: &[Arc<dyn BaseTool>]) -> PromptTemplate {
        let tool_descriptions: String = tools
            .iter()
            .map(|t| format!("{}: {}", t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n");
        let tool_names: Vec<String> = tools.iter().map(|t| t.name().to_string()).collect();
        let tool_names_str = tool_names.join(", ");

        let template = format!(
            "You are a MRKL system (Modular Reasoning, Knowledge and Language).\n\n\
            You have access to the following modules (tools):\n\
            {tool_descriptions}\n\n\
            Use the following format:\n\n\
            Question: the input question you must answer\n\
            Thought: you should always think about what to do\n\
            Action: the action to take, should be one of [{tool_names_str}]\n\
            Action Input: the input to the action\n\
            Observation: the result of the action\n\
            ... (this Thought/Action/Action Input/Observation can repeat N times)\n\
            Thought: I now know the final answer\n\
            Final Answer: the final answer to the original input question\n\n\
            Begin!\n\n\
            Question: {{input}}\n\
            Thought: {{agent_scratchpad}}"
        );

        PromptTemplate::from_template(&template)
    }

    fn parse_output(text: &str) -> Result<AgentNextStep> {
        let final_re = Regex::new(r"(?s)Final Answer:\s*(.*)")
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

        let action_re = Regex::new(
            r"Thought:\s*(.*?)\n\s*Action:\s*(\w+)\n\s*Action Input:\s*(.*?)(?:\n|$)",
        )
        .map_err(|e| ChainError::ParserError(e.to_string()))?;

        if let Some(caps) = action_re.captures(text) {
            let _thought = caps.get(1).map_or("", |m| m.as_str().trim());
            let tool = caps.get(2).map_or("", |m| m.as_str().trim());
            let input = caps.get(3).map_or("", |m| m.as_str().trim());
            return Ok(AgentNextStep::Action(AgentAction {
                tool: tool.to_string(),
                tool_input: input.to_string(),
                log: text.to_string(),
            }));
        }

        Err(ChainError::ParserError(format!(
            "Could not parse MRKL output: {}",
            text
        )))
    }

    fn construct_scratchpad(&self, steps: &[IntermediateStep]) -> String {
        let mut parts = Vec::new();
        for step in steps {
            parts.push(format!(
                "Thought: {}\nAction: {}\nAction Input: {}\nObservation: {}",
                step.action.log, step.action.tool, step.action.tool_input, step.observation
            ));
        }
        parts.push("Thought: ".to_string());
        parts.join("\n")
    }
}

#[async_trait]
impl Agent for MRKLAgent {
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

        let messages = vec![
            SystemMessage::new("You are a MRKL agent. Use your modules to reason and act.").into(),
            HumanMessage::new(&prompt).into(),
        ];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        Self::parse_output(&response.content)
    }

    async fn return_stopped_response(
        &self,
        early_stopping_method: &str,
        intermediate_steps: &[IntermediateStep],
        inputs: &HashMap<String, Value>,
    ) -> Result<AgentFinish> {
        match early_stopping_method {
            "force" => {
                let mut return_values = HashMap::new();
                return_values.insert(
                    "output".to_string(),
                    Value::String(
                        intermediate_steps
                            .last()
                            .map_or_else(|| "MRKL agent stopped".to_string(), |s| {
                                s.observation.clone()
                            }),
                    ),
                );
                Ok(AgentFinish {
                    return_values,
                    log: "MRKL agent stopped due to force early stopping".to_string(),
                })
            }
            "generate" => {
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
                let messages = vec![
                    SystemMessage::new("Given the observations, provide a final answer.").into(),
                    HumanMessage::new(&prompt).into(),
                ];
                let response = self.llm.predict_messages(&messages, None, None).await?;
                let content = response.content;

                let mut return_values = HashMap::new();
                return_values.insert("output".to_string(), Value::String(content.clone()));
                Ok(AgentFinish {
                    return_values,
                    log: content,
                })
            }
            _ => {
                let mut return_values = HashMap::new();
                return_values.insert(
                    "output".to_string(),
                    Value::String("MRKL agent stopped.".to_string()),
                );
                Ok(AgentFinish {
                    return_values,
                    log: format!("MRKL agent stopped via method: {}", early_stopping_method),
                })
            }
        }
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
