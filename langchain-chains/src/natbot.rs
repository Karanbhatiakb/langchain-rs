use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct NatBotChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    max_iterations: usize,
    verbose: bool,
}

impl NatBotChain {
    pub fn new(llm: Arc<dyn ChatModel>) -> Self {
        let prompt = PromptTemplate::from_template(
            "You are a web navigation assistant. Given an objective and the current browser state, \
            output the next action to take.\n\n\
            Objective: {objective}\n\n\
            Browser State:\n{browser_state}\n\n\
            Previous actions:\n{previous_actions}\n\n\
            Output the next action as JSON:\n\
            {{\"action\": \"click|type|scroll|navigate|wait|done\", \"selector\": \"...\", \"value\": \"...\"}}\n\n\
            Next action:",
        );
        Self {
            llm,
            prompt,
            max_iterations: 10,
            verbose: false,
        }
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for NatBotChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["objective".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let objective = inputs
            .get("objective")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let browser_state = inputs
            .get("browser_state")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let previous_actions = inputs
            .get("previous_actions")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if self.verbose {
            info!("NatBotChain objective: {}", objective);
        }

        let mut kwargs = HashMap::new();
        kwargs.insert("objective".to_string(), objective);
        kwargs.insert("browser_state".to_string(), browser_state);
        kwargs.insert("previous_actions".to_string(), previous_actions);
        let formatted = self.prompt.format(&kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&formatted).into()], None, None)
            .await?;

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(response.content));
        Ok(result)
    }
}
