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

pub struct ExtractionChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    schema: Value,
    verbose: bool,
}

impl ExtractionChain {
    pub fn new(llm: Arc<dyn ChatModel>, schema: Value) -> Self {
        let prompt = PromptTemplate::from_template(
            "Extract structured information from the text according to the schema.\n\n\
            Text: {input}\n\n\
            Schema:\n{schema}\n\n\
            Extracted JSON:",
        );
        Self {
            llm,
            prompt,
            schema,
            verbose: false,
        }
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for ExtractionChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["extracted".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if self.verbose {
            info!("ExtractionChain input: {}", input);
        }

        let schema_str =
            serde_json::to_string_pretty(&self.schema).unwrap_or_else(|_| "{}".to_string());

        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input);
        kwargs.insert("schema".to_string(), schema_str);
        let formatted = self.prompt.format(&kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&formatted).into()], None, None)
            .await?;

        let mut result = HashMap::new();
        result.insert("extracted".to_string(), Value::String(response.content));
        Ok(result)
    }
}
