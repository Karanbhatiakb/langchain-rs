//! Plan-and-execute agent implementation with two-phase execution.

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

pub struct PlanAndExecuteAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    plan_prompt: PromptTemplate,
    execute_prompt: PromptTemplate,
    max_steps: usize,
}

#[derive(Debug, Clone)]
struct PlanStep {
    description: String,
    tool_name: Option<String>,
    tool_input: Option<String>,
}

impl PlanAndExecuteAgent {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        tools: Vec<Arc<dyn BaseTool>>,
        plan_prompt: Option<PromptTemplate>,
        execute_prompt: Option<PromptTemplate>,
    ) -> Self {
        let plan_prompt =
            plan_prompt.unwrap_or_else(|| Self::default_plan_prompt(&tools));
        let execute_prompt =
            execute_prompt.unwrap_or_else(|| Self::default_execute_prompt(&tools));
        Self {
            llm,
            tools,
            plan_prompt,
            execute_prompt,
            max_steps: 10,
        }
    }

    pub fn with_max_steps(mut self, n: usize) -> Self {
        self.max_steps = n;
        self
    }

    fn default_plan_prompt(tools: &[Arc<dyn BaseTool>]) -> PromptTemplate {
        let tool_descriptions: String = tools
            .iter()
            .map(|t| format!("- {}: {}", t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n");
        let tool_names: Vec<String> = tools.iter().map(|t| t.name().to_string()).collect();
        let tool_names_str = tool_names.join(", ");

        let template = format!(
            "You are a planning agent. Given an objective, create a step-by-step plan.\n\n\
            Available tools: {tool_names_str}\n\n\
            Tool descriptions:\n\
            {tool_descriptions}\n\n\
            For each step, output one of:\n\
            Step N: <description of what to do>\n\
            Step N: Use tool <tool_name> with input <tool_input>\n\n\
            Objective: {{input}}\n\n\
            Plan:"
        );

        PromptTemplate::from_template(&template)
    }

    fn default_execute_prompt(tools: &[Arc<dyn BaseTool>]) -> PromptTemplate {
        let tool_names: Vec<String> = tools.iter().map(|t| t.name().to_string()).collect();
        let tool_names_str = tool_names.join(", ");

        let template = format!(
            "You are an execution agent. Execute the current step of the plan.\n\n\
            Available tools: {tool_names_str}\n\n\
            Current step: {{current_step}}\n\
            Previous observations: {{previous_observations}}\n\n\
            Respond with either:\n\
            Action: <tool_name>\n\
            Action Input: <input>\n\n\
            Or if the step is complete:\n\
            Final Answer: <result>"
        );

        PromptTemplate::from_template(&template)
    }

    fn parse_plan(&self, text: &str) -> Vec<PlanStep> {
        let step_re = Regex::new(r"(?i)Step\s+\d+:\s*(.+)").unwrap();
        let tool_re = Regex::new(r"(?i)Use tool\s+(\w+)\s+with input\s+(.+)").unwrap();

        let mut steps = Vec::new();
        for cap in step_re.captures_iter(text) {
            let description = cap.get(1).unwrap().as_str().trim().to_string();
            if let Some(tool_cap) = tool_re.captures(&description) {
                steps.push(PlanStep {
                    description: description.clone(),
                    tool_name: Some(tool_cap.get(1).unwrap().as_str().trim().to_string()),
                    tool_input: Some(tool_cap.get(2).unwrap().as_str().trim().to_string()),
                });
            } else {
                steps.push(PlanStep {
                    description,
                    tool_name: None,
                    tool_input: None,
                });
            }
        }

        if steps.is_empty() {
            steps.push(PlanStep {
                description: text.trim().to_string(),
                tool_name: None,
                tool_input: None,
            });
        }

        steps
    }

    fn parse_execute_output(&self, text: &str) -> Result<AgentNextStep> {
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

        let action_re = Regex::new(r"Action:\s*(\w+)\s*\n\s*Action Input:\s*(.*?)(?:\n|$)")
            .map_err(|e| ChainError::ParserError(e.to_string()))?;
        if let Some(caps) = action_re.captures(text) {
            let tool = caps.get(1).unwrap().as_str().trim().to_string();
            let input = caps.get(2).unwrap().as_str().trim().to_string();
            return Ok(AgentNextStep::Action(AgentAction {
                tool,
                tool_input: input,
                log: text.to_string(),
            }));
        }

        let mut return_values = HashMap::new();
        return_values.insert("output".to_string(), Value::String(text.trim().to_string()));
        Ok(AgentNextStep::Finish(AgentFinish {
            return_values,
            log: text.to_string(),
        }))
    }
}

#[async_trait]
impl Agent for PlanAndExecuteAgent {
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
        let input_str = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if intermediate_steps.is_empty() {
            let mut kwargs = HashMap::new();
            kwargs.insert("input".to_string(), input_str.clone());
            let plan_prompt = self.plan_prompt.format(&kwargs)?;

            let messages = vec![
                SystemMessage::new("You are a planning agent. Create a step-by-step plan.").into(),
                HumanMessage::new(&plan_prompt).into(),
            ];
            let response = self.llm.predict_messages(&messages, None, None).await?;
            let plan_text = response.content;

            let steps = self.parse_plan(&plan_text);
            if let Some(first_step) = steps.first() {
                if let (Some(tool_name), Some(tool_input)) =
                    (&first_step.tool_name, &first_step.tool_input)
                {
                    return Ok(AgentNextStep::Action(AgentAction {
                        tool: tool_name.clone(),
                        tool_input: tool_input.clone(),
                        log: format!("Plan step 1: {}", first_step.description),
                    }));
                }

                let previous_observations = String::new();
                let mut exec_kwargs = HashMap::new();
                exec_kwargs.insert("current_step".to_string(), first_step.description.clone());
                exec_kwargs.insert("previous_observations".to_string(), previous_observations);
                let exec_prompt = self.execute_prompt.format(&exec_kwargs)?;

                let messages = vec![
                    SystemMessage::new("Execute the current step of the plan using available tools.").into(),
                    HumanMessage::new(&exec_prompt).into(),
                ];
                let exec_response = self.llm.predict_messages(&messages, None, None).await?;
                return self.parse_execute_output(&exec_response.content);
            }
        }

        let observations: String = intermediate_steps
            .iter()
            .map(|s| format!("{}: {}", s.action.tool, s.observation))
            .collect::<Vec<_>>()
            .join("\n");

        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input_str.clone());
        kwargs.insert("previous_observations".to_string(), observations);

        let combined_prompt = format!(
            "Objective: {}\n\nPrevious observations:\n{}\n\nBased on the observations so far, what should we do next? Respond with a Final Answer or an Action.",
            inputs.get("input").and_then(|v| v.as_str()).unwrap_or(""),
            intermediate_steps.iter().map(|s| format!("- {}: {}", s.action.tool, s.observation)).collect::<Vec<_>>().join("\n")
        );

        let messages = vec![
            SystemMessage::new("Continue executing the plan based on observations.").into(),
            HumanMessage::new(&combined_prompt).into(),
        ];
        let response = self.llm.predict_messages(&messages, None, None).await?;
        self.parse_execute_output(&response.content)
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
            .map_or_else(|| "Plan execution stopped.".to_string(), |s| s.observation.clone());
        return_values.insert("output".to_string(), Value::String(output));
        Ok(AgentFinish {
            return_values,
            log: "Plan execution stopped due to max iterations".to_string(),
        })
    }

    fn create_prompt(&self) -> Result<PromptTemplate> {
        Ok(self.plan_prompt.clone())
    }

    fn tools(&self) -> Vec<Arc<dyn BaseTool>> {
        self.tools.clone()
    }

    fn llm(&self) -> Arc<dyn ChatModel> {
        self.llm.clone()
    }
}
