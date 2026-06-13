//! Multi-agent implementation with agent routing.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::Agent;
use crate::types::{AgentFinish, AgentNextStep, IntermediateStep};
use langchain_tools::traits::BaseTool;

pub struct AgentRouter {
    llm: Arc<dyn ChatModel>,
    agents: HashMap<String, Arc<dyn Agent>>,
    router_prompt: PromptTemplate,
}

impl AgentRouter {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        agents: HashMap<String, Arc<dyn Agent>>,
    ) -> Self {
        let agent_names: Vec<String> = agents.keys().cloned().collect();
        let agent_descriptions: String = agents
            .iter()
            .map(|(name, _)| format!("- {}", name))
            .collect::<Vec<_>>()
            .join("\n");

        let template = format!(
            "You are a router that selects the best agent for a given task.\n\n\
            Available agents:\n\
            {agent_descriptions}\n\n\
            Respond with ONLY the agent name from: [{}]\n\n\
            Input: {{input}}\n\
            Agent:",
            agent_names.join(", ")
        );

        Self {
            llm,
            agents,
            router_prompt: PromptTemplate::from_template(&template),
        }
    }

    pub fn with_router_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.router_prompt = prompt;
        self
    }

    pub async fn route(&self, input: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input.to_string());
        let prompt = self.router_prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let selected = response.content.trim().to_lowercase();
        if self.agents.contains_key(&selected) {
            Ok(selected)
        } else {
            for name in self.agents.keys() {
                if selected.contains(name.to_lowercase().as_str()) {
                    return Ok(name.clone());
                }
            }
            self.agents
                .keys()
                .next()
                .cloned()
                .ok_or_else(|| ChainError::AgentError("No agents available".to_string()))
        }
    }

    pub fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>> {
        self.agents.get(name).cloned()
    }

    pub fn agent_names(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }
}

pub struct MultiAgent {
    llm: Arc<dyn ChatModel>,
    tools: Vec<Arc<dyn BaseTool>>,
    router: AgentRouter,
    default_agent_name: Option<String>,
}

impl MultiAgent {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        tools: Vec<Arc<dyn BaseTool>>,
        agents: HashMap<String, Arc<dyn Agent>>,
    ) -> Self {
        let router = AgentRouter::new(llm.clone(), agents);
        Self {
            llm,
            tools,
            router,
            default_agent_name: None,
        }
    }

    pub fn with_default_agent(mut self, name: impl Into<String>) -> Self {
        self.default_agent_name = Some(name.into());
        self
    }

    pub fn with_router_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.router = AgentRouter::new(self.llm.clone(), self.router.agents.clone())
            .with_router_prompt(prompt);
        self
    }

    pub fn router(&self) -> &AgentRouter {
        &self.router
    }
}

#[async_trait]
impl Agent for MultiAgent {
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
            let agent_name = self.router.route(&input_str).await?;

            if let Some(agent) = self.router.get_agent(&agent_name) {
                let mut routed_inputs = inputs.clone();
                routed_inputs.insert(
                    "_routed_agent".to_string(),
                    Value::String(agent_name),
                );
                return agent.plan(intermediate_steps, &routed_inputs).await;
            }
        }

        if !intermediate_steps.is_empty() {
            let agent_name = inputs
                .get("_routed_agent")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if !agent_name.is_empty() {
                if let Some(agent) = self.router.get_agent(agent_name) {
                    return agent.plan(intermediate_steps, inputs).await;
                }
            }
        }

        if let Some(ref default_name) = self.default_agent_name {
            if let Some(agent) = self.router.get_agent(default_name) {
                return agent.plan(intermediate_steps, inputs).await;
            }
        }

        let messages = vec![
            SystemMessage::new("You are a multi-agent orchestrator. Provide a direct answer or route to an agent.").into(),
            HumanMessage::new(&input_str).into(),
        ];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let mut return_values = HashMap::new();
        return_values.insert("output".to_string(), Value::String(response.content.clone()));
        Ok(AgentNextStep::Finish(AgentFinish {
            return_values,
            log: response.content,
        }))
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
            .map_or_else(|| "Multi-agent execution stopped.".to_string(), |s| {
                s.observation.clone()
            });
        return_values.insert("output".to_string(), Value::String(output));
        Ok(AgentFinish {
            return_values,
            log: "Multi-agent execution stopped".to_string(),
        })
    }

    fn create_prompt(&self) -> Result<PromptTemplate> {
        Ok(self.router.router_prompt.clone())
    }

    fn tools(&self) -> Vec<Arc<dyn BaseTool>> {
        self.tools.clone()
    }

    fn llm(&self) -> Arc<dyn ChatModel> {
        self.llm.clone()
    }
}
