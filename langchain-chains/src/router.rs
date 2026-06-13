//! Router chain implementation.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct RouterChain {
    llm: Arc<dyn ChatModel>,
    router_prompt: String,
    destination_chains: HashMap<String, Arc<dyn Chain>>,
    default_chain: Option<Arc<dyn Chain>>,
    input_variables: Vec<String>,
    verbose: bool,
}

impl RouterChain {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        router_prompt: impl Into<String>,
        destination_chains: HashMap<String, Arc<dyn Chain>>,
    ) -> Self {
        let input_variables = vec!["input".to_string()];
        Self {
            llm,
            router_prompt: router_prompt.into(),
            destination_chains,
            default_chain: None,
            input_variables,
            verbose: false,
        }
    }

    pub fn with_default_chain(mut self, chain: Arc<dyn Chain>) -> Self {
        self.default_chain = Some(chain);
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    async fn select_destination(&self, input: &str) -> Result<String> {
        let prompt = format!("{}\n\nInput: {}", self.router_prompt, input);

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&prompt).into()], None, None)
            .await?;

        let destination = response.content.trim().to_lowercase();
        Ok(destination)
    }
}

#[async_trait]
impl Chain for RouterChain {
    fn input_keys(&self) -> Vec<String> {
        self.input_variables.clone()
    }

    fn output_keys(&self) -> Vec<String> {
        let mut keys = vec!["destination".to_string(), "response".to_string()];
        for chain in self.destination_chains.values() {
            keys.extend(chain.output_keys());
        }
        keys
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let input_str = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("RouterChain routing input: {}", input_str);
        }

        let destination = self.select_destination(&input_str).await?;

        let mut result = HashMap::new();
        result.insert("destination".to_string(), Value::String(destination.clone()));

        if let Some(chain) = self.destination_chains.get(&destination) {
            let chain_output = chain.call(inputs).await?;
            result.extend(chain_output);
        } else if let Some(ref default) = self.default_chain {
            let chain_output = default.call(inputs).await?;
            result.extend(chain_output);
        } else {
            result.insert(
                "response".to_string(),
                Value::String(format!("No chain found for destination: {}", destination)),
            );
        }

        Ok(result)
    }
}

pub struct MultiRouteChain {
    router_chain: Arc<RouterChain>,
    destination_chains: HashMap<String, Arc<dyn Chain>>,
    default_chain: Option<Arc<dyn Chain>>,
}

impl MultiRouteChain {
    pub fn new(
        router_chain: Arc<RouterChain>,
        destination_chains: HashMap<String, Arc<dyn Chain>>,
    ) -> Self {
        Self {
            router_chain,
            destination_chains,
            default_chain: None,
        }
    }

    pub fn with_default_chain(mut self, chain: Arc<dyn Chain>) -> Self {
        self.default_chain = Some(chain);
        self
    }
}

#[async_trait]
impl Chain for MultiRouteChain {
    fn input_keys(&self) -> Vec<String> {
        self.router_chain.input_keys()
    }

    fn output_keys(&self) -> Vec<String> {
        let mut keys = vec!["destination".to_string(), "response".to_string()];
        for chain in self.destination_chains.values() {
            keys.extend(chain.output_keys());
        }
        keys
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let routing = self.router_chain.call(inputs.clone()).await?;
        let destination = routing
            .get("destination")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut result = HashMap::new();
        result.insert("destination".to_string(), Value::String(destination.clone()));

        if let Some(chain) = self.destination_chains.get(&destination) {
            let chain_output = chain.call(inputs).await?;
            result = chain_output;
            result.insert("destination".to_string(), Value::String(destination));
        } else if let Some(ref default) = self.default_chain {
            let chain_output = default.call(inputs).await?;
            result = chain_output;
        }

        Ok(result)
    }
}
