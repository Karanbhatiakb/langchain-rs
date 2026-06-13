//! Core [`Agent`] trait for planning and executing tool-based actions.

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use langchain_tools::traits::BaseTool;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{AgentFinish, AgentNextStep, IntermediateStep};

/// Trait for agents that plan and execute tool-based actions.
///
/// Agents observe intermediate steps, decide on the next action (tool call or
/// finish), and can create their own prompt and return stopped responses.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the input key names expected by this agent.
    fn input_keys(&self) -> Vec<String>;
    /// Returns the output key names produced by this agent.
    fn output_keys(&self) -> Vec<String>;

    /// Plans the next step (action or finish) given intermediate steps and
    /// inputs.
    async fn plan(
        &self,
        intermediate_steps: &[IntermediateStep],
        inputs: &HashMap<String, Value>,
    ) -> Result<AgentNextStep>;

    /// Returns a finish response when early stopping is triggered.
    async fn return_stopped_response(
        &self,
        early_stopping_method: &str,
        intermediate_steps: &[IntermediateStep],
        inputs: &HashMap<String, Value>,
    ) -> Result<AgentFinish>;

    /// Creates the prompt template used by this agent.
    fn create_prompt(&self) -> Result<PromptTemplate>;
    /// Returns the tools available to this agent.
    fn tools(&self) -> Vec<Arc<dyn BaseTool>>;
    /// Returns the LLM used by this agent.
    fn llm(&self) -> Arc<dyn ChatModel>;
}
