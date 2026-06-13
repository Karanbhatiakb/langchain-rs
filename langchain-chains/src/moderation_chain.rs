//! Moderation chain implementation — checks input for harmful content.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct ModerationResult {
    pub flagged: bool,
    pub categories: Vec<String>,
    pub category_scores: HashMap<String, f64>,
}

#[async_trait]
pub trait ModerationApi: Send + Sync {
    async fn moderate(&self, input: &str) -> Result<ModerationResult>;
}

pub struct KeywordModerationApi {
    keywords: HashMap<String, Vec<String>>,
}

impl KeywordModerationApi {
    pub fn new() -> Self {
        let mut keywords = HashMap::new();
        keywords.insert(
            "violence".to_string(),
            vec![
                "kill".to_string(),
                "murder".to_string(),
                "attack".to_string(),
                "assault".to_string(),
                "harm".to_string(),
            ],
        );
        keywords.insert(
            "hate".to_string(),
            vec![
                "hate".to_string(),
                "bigot".to_string(),
                "racist".to_string(),
                "sexist".to_string(),
            ],
        );
        keywords.insert(
            "harassment".to_string(),
            vec![
                "harass".to_string(),
                "bully".to_string(),
                "threat".to_string(),
                "intimidate".to_string(),
            ],
        );
        keywords.insert(
            "self_harm".to_string(),
            vec![
                "suicide".to_string(),
                "self-harm".to_string(),
                "self harm".to_string(),
            ],
        );
        keywords.insert(
            "sexual".to_string(),
            vec![
                "sexual".to_string(),
                "explicit".to_string(),
                "pornographic".to_string(),
            ],
        );

        Self { keywords }
    }

    pub fn with_keywords(mut self, category: impl Into<String>, words: Vec<String>) -> Self {
        self.keywords.insert(category.into(), words);
        self
    }
}

impl Default for KeywordModerationApi {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModerationApi for KeywordModerationApi {
    async fn moderate(&self, input: &str) -> Result<ModerationResult> {
        let lower_input = input.to_lowercase();
        let mut flagged = false;
        let mut categories = Vec::new();
        let mut category_scores = HashMap::new();

        for (category, words) in &self.keywords {
            let mut match_count = 0;
            for word in words {
                if lower_input.contains(&word.to_lowercase()) {
                    match_count += 1;
                }
            }

            let score = if words.is_empty() {
                0.0
            } else {
                match_count as f64 / words.len() as f64
            };

            if match_count > 0 {
                flagged = true;
                categories.push(category.clone());
            }

            category_scores.insert(category.clone(), score);
        }

        Ok(ModerationResult {
            flagged,
            categories,
            category_scores,
        })
    }
}

pub struct ModerationChain {
    llm: Option<Arc<dyn ChatModel>>,
    moderation_api: Arc<dyn ModerationApi>,
    prompt: Option<PromptTemplate>,
    throw_error: bool,
    verbose: bool,
}

impl ModerationChain {
    pub fn new(moderation_api: Arc<dyn ModerationApi>) -> Self {
        Self {
            llm: None,
            moderation_api,
            prompt: None,
            throw_error: false,
            verbose: false,
        }
    }

    pub fn with_llm(mut self, llm: Arc<dyn ChatModel>) -> Self {
        self.llm = Some(llm);
        self
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn with_throw_error(mut self, throw: bool) -> Self {
        self.throw_error = throw;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn default_with_keywords() -> Self {
        Self::new(Arc::new(KeywordModerationApi::new()))
    }
}

#[async_trait]
impl Chain for ModerationChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string(), "flagged".to_string(), "categories".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("ModerationChain checking input of length {}", input.len());
        }

        let result = self.moderation_api.moderate(&input).await?;

        if self.verbose && result.flagged {
            info!(
                "ModerationChain flagged input in categories: {:?}",
                result.categories
            );
        }

        if result.flagged && self.throw_error {
            return Err(ChainError::ValidationError(format!(
                "Input flagged for moderation: {}",
                result.categories.join(", ")
            )));
        }

        let output = if result.flagged {
            format!(
                "This content was flagged for: {}",
                result.categories.join(", ")
            )
        } else {
            input.clone()
        };

        let mut result_map = HashMap::new();
        result_map.insert("output".to_string(), Value::String(output));
        result_map.insert("flagged".to_string(), Value::Bool(result.flagged));
        result_map.insert(
            "categories".to_string(),
            serde_json::to_value(&result.categories).unwrap_or_default(),
        );

        Ok(result_map)
    }
}
