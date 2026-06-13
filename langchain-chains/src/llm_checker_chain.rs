//! LLM Checker chain implementation — verifies and critiques LLM outputs.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct LLMCheckerChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    critique_prompt: PromptTemplate,
    revision_prompt: PromptTemplate,
    max_revisions: usize,
    verbose: bool,
}

impl LLMCheckerChain {
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        let critique_prompt = PromptTemplate::from_template(
            "You are a fact-checker. Given a question and an answer, verify the answer.\n\n\
            Question: {question}\n\n\
            Answer: {answer}\n\n\
            Check the following:\n\
            1. Is the answer factually correct?\n\
            2. Is the answer complete?\n\
            3. Are there any logical errors?\n\n\
            Respond with JSON:\n\
            {{\"correct\": true/false, \"critique\": \"explanation\", \"assertions\": [\"assertion1\", \"assertion2\"]}}",
        );
        let revision_prompt = PromptTemplate::from_template(
            "You are a reviser. Given a question, an answer, and a critique, provide a revised answer.\n\n\
            Question: {question}\n\n\
            Original Answer: {answer}\n\n\
            Critique: {critique}\n\n\
            Revised Answer:",
        );

        Self {
            llm,
            prompt,
            critique_prompt,
            revision_prompt,
            max_revisions: 2,
            verbose: false,
        }
    }

    pub fn with_critique_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.critique_prompt = prompt;
        self
    }

    pub fn with_revision_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.revision_prompt = prompt;
        self
    }

    pub fn with_max_revisions(mut self, n: usize) -> Self {
        self.max_revisions = n;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    async fn generate_initial_answer(&self, question: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("question".to_string(), question.to_string());
        let formatted = self.prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&formatted).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;
        Ok(response.content)
    }

    async fn critique_answer(&self, question: &str, answer: &str) -> Result<CritiqueResult> {
        let mut kwargs = HashMap::new();
        kwargs.insert("question".to_string(), question.to_string());
        kwargs.insert("answer".to_string(), answer.to_string());
        let formatted = self.critique_prompt.format(&kwargs)?;

        let messages = vec![
            SystemMessage::new("You are a fact-checker. Respond in JSON format.").into(),
            HumanMessage::new(&formatted).into(),
        ];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let content = response.content.trim();
        let json_start = content.find('{').ok_or_else(|| {
            ChainError::ParserError("No JSON in critique response".to_string())
        })?;
        let json_end = content.rfind('}').ok_or_else(|| {
            ChainError::ParserError("Unclosed JSON in critique response".to_string())
        })?;
        let json_str = &content[json_start..=json_end];

        let parsed: Value = serde_json::from_str(json_str)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse critique JSON: {}", e)))?;

        let correct = parsed
            .get("correct")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let critique = parsed
            .get("critique")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let assertions = parsed
            .get("assertions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(CritiqueResult {
            correct,
            critique,
            assertions,
        })
    }

    async fn revise_answer(
        &self,
        question: &str,
        answer: &str,
        critique: &str,
    ) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("question".to_string(), question.to_string());
        kwargs.insert("answer".to_string(), answer.to_string());
        kwargs.insert("critique".to_string(), critique.to_string());
        let formatted = self.revision_prompt.format(&kwargs)?;

        let messages = vec![
            SystemMessage::new("You are a reviser. Provide a corrected answer based on the critique.").into(),
            HumanMessage::new(&formatted).into(),
        ];
        let response = self.llm.predict_messages(&messages, None, None).await?;
        Ok(response.content)
    }

    async fn check_assertions(&self, assertions: &[String]) -> Result<bool> {
        for assertion in assertions {
            let mut kwargs = HashMap::new();
            kwargs.insert("assertion".to_string(), assertion.clone());
            let prompt = format!(
                "Is the following assertion factually correct? Respond with only 'true' or 'false'.\n\nAssertion: {}",
                assertion
            );
            let messages = vec![HumanMessage::new(&prompt).into()];
            let response = self.llm.predict_messages(&messages, None, None).await?;
            let answer = response.content.trim().to_lowercase();
            if answer.starts_with('f') || answer.contains("false") {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

struct CritiqueResult {
    correct: bool,
    critique: String,
    assertions: Vec<String>,
}

#[async_trait]
impl Chain for LLMCheckerChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("LLMCheckerChain processing: {}", question);
        }

        let mut current_answer = self.generate_initial_answer(&question).await?;

        for revision in 0..=self.max_revisions {
            if self.verbose {
                info!("LLMCheckerChain revision {}/{}", revision, self.max_revisions);
            }

            let critique = self.critique_answer(&question, &current_answer).await?;

            if critique.correct && self.check_assertions(&critique.assertions).await? {
                if self.verbose {
                    info!("LLMCheckerChain answer verified as correct");
                }
                let mut result = HashMap::new();
                result.insert("output".to_string(), Value::String(current_answer));
                return Ok(result);
            }

            if revision < self.max_revisions {
                if self.verbose {
                    info!("LLMCheckerChain revising answer based on critique: {}", critique.critique);
                }
                current_answer = self
                    .revise_answer(&question, &current_answer, &critique.critique)
                    .await?;
            }
        }

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(current_answer));
        Ok(result)
    }
}

pub struct LLMSummarizationCheckerChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    max_revisions: usize,
    verbose: bool,
}

impl LLMSummarizationCheckerChain {
    pub fn new(llm: Arc<dyn ChatModel>) -> Self {
        let prompt = PromptTemplate::from_template(
            "You are a summarization checker. Given a text and its summary, verify the summary is correct and complete.\n\n\
            Original Text: {text}\n\n\
            Summary: {summary}\n\n\
            Check the following:\n\
            1. Is the summary factually accurate?\n\
            2. Does the summary capture all key points?\n\
            3. Are there any omissions or errors?\n\n\
            Respond with JSON:\n\
            {{\"correct\": true/false, \"issues\": [\"issue1\", \"issue2\"], \"revised_summary\": \"...\"}}",
        );
        Self {
            llm,
            prompt,
            max_revisions: 2,
            verbose: false,
        }
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_max_revisions(mut self, n: usize) -> Self {
        self.max_revisions = n;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for LLMSummarizationCheckerChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["text".to_string(), "summary".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let text = inputs
            .get("text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let summary = inputs
            .get("summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if self.verbose {
            info!("LLMSummarizationCheckerChain checking summary");
        }

        let mut current_summary = summary;
        for revision in 0..=self.max_revisions {
            if self.verbose {
                info!(
                    "LLMSummarizationCheckerChain revision {}/{}",
                    revision, self.max_revisions
                );
            }

            let mut kwargs = HashMap::new();
            kwargs.insert("text".to_string(), text.clone());
            kwargs.insert("summary".to_string(), current_summary.clone());
            let formatted = self.prompt.format(&kwargs)?;

            let response = self
                .llm
                .predict_messages(&[HumanMessage::new(&formatted).into()], None, None)
                .await?;

            let parsed: Value = serde_json::from_str(&response.content)
                .unwrap_or_else(|_| Value::Null);

            let correct = parsed
                .get("correct")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if correct {
                if self.verbose {
                    info!("LLMSummarizationCheckerChain summary verified as correct");
                }
                let mut result = HashMap::new();
                result.insert("output".to_string(), Value::String(current_summary));
                return Ok(result);
            }

            if revision < self.max_revisions {
                if let Some(revised) = parsed
                    .get("revised_summary")
                    .and_then(|v| v.as_str())
                {
                    current_summary = revised.to_string();
                }
            }
        }

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(current_summary));
        Ok(result)
    }
}
