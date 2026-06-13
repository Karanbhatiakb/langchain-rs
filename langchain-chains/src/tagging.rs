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

pub struct TaggingChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    tags: Vec<String>,
    verbose: bool,
}

impl TaggingChain {
    pub fn new(llm: Arc<dyn ChatModel>, tags: Vec<String>) -> Self {
        let prompt = PromptTemplate::from_template(
            "Classify the following text and assign one or more of the given tags.\n\n\
            Text: {input}\n\n\
            Available tags: {tags}\n\n\
            Respond with a JSON object containing:\n\
            - \"tags\": array of assigned tag strings\n\
            - \"reasoning\": brief explanation for each tag\n\n\
            JSON:",
        );
        Self {
            llm,
            prompt,
            tags,
            verbose: false,
        }
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for TaggingChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["tags".to_string(), "reasoning".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if self.verbose {
            info!("TaggingChain input: {}", input);
        }

        let tags_str = self.tags.join(", ");

        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input);
        kwargs.insert("tags".to_string(), tags_str);
        let formatted = self.prompt.format(&kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&formatted).into()], None, None)
            .await?;

        let parsed: Value = serde_json::from_str(&response.content)
            .unwrap_or_else(|_| Value::Object(serde_json::Map::new()));

        let result_tags = parsed
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let reasoning = parsed
            .get("reasoning")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let mut result = HashMap::new();
        result.insert(
            "tags".to_string(),
            Value::Array(result_tags.into_iter().map(Value::String).collect()),
        );
        result.insert("reasoning".to_string(), Value::String(reasoning));
        Ok(result)
    }
}
