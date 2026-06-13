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

pub struct ErnieFunctionsChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    function_name: String,
    function_description: String,
    verbose: bool,
}

impl ErnieFunctionsChain {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        function_name: impl Into<String>,
        function_description: impl Into<String>,
    ) -> Self {
        let prompt = PromptTemplate::from_template(
            "Extract the requested information from the input.\n\n\
            Input: {input}\n\n\
            Function: {function_name}\n\
            Description: {function_description}\n\n\
            Response as JSON:",
        );
        Self {
            llm,
            prompt,
            function_name: function_name.into(),
            function_description: function_description.into(),
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
impl Chain for ErnieFunctionsChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if self.verbose {
            info!("ErnieFunctionsChain input: {}", input);
        }

        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input);
        kwargs.insert(
            "function_name".to_string(),
            self.function_name.clone(),
        );
        kwargs.insert(
            "function_description".to_string(),
            self.function_description.clone(),
        );
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
