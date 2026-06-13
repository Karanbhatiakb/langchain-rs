//! Constitutional chain implementation — uses constitutional principles to evaluate and revise outputs.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalPrinciple {
    pub name: String,
    pub critique_request: String,
    pub revision_request: String,
}

impl ConstitutionalPrinciple {
    pub fn new(
        name: impl Into<String>,
        critique_request: impl Into<String>,
        revision_request: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            critique_request: critique_request.into(),
            revision_request: revision_request.into(),
        }
    }

    pub fn harmful_principle() -> Self {
        Self::new(
            "harmful",
            "Identify any harmful, unethical, racist, sexist, or dangerous content in the response.",
            "Rewrite the response to remove any harmful, unethical, racist, sexist, or dangerous content while preserving the useful information.",
        )
    }

    pub fn offensive_principle() -> Self {
        Self::new(
            "offensive",
            "Identify any offensive, insulting, or disrespectful content in the response.",
            "Rewrite the response to be respectful and non-offensive while preserving the useful information.",
        )
    }

    pub fn privacy_principle() -> Self {
        Self::new(
            "privacy",
            "Identify any personal information (names, addresses, phone numbers, emails, SSNs) in the response.",
            "Rewrite the response to remove or anonymize any personal information while preserving the useful information.",
        )
    }

    pub fn accuracy_principle() -> Self {
        Self::new(
            "accuracy",
            "Identify any factually incorrect or misleading claims in the response.",
            "Rewrite the response to correct any factual errors or remove misleading claims.",
        )
    }
}

pub struct ConstitutionalChain {
    llm: Arc<dyn ChatModel>,
    chain: Arc<dyn Chain>,
    principles: Vec<ConstitutionalPrinciple>,
    critique_prompt: PromptTemplate,
    revision_prompt: PromptTemplate,
    verbose: bool,
}

impl ConstitutionalChain {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        chain: Arc<dyn Chain>,
        principles: Vec<ConstitutionalPrinciple>,
    ) -> Self {
        let critique_prompt = PromptTemplate::from_template(
            "Given the following response, identify any violations of the principle.\n\n\
            Principle: {principle}\n\n\
            Response: {response}\n\n\
            Critique:",
        );
        let revision_prompt = PromptTemplate::from_template(
            "Given the following response, critique, and revision request, provide a revised response.\n\n\
            Original Response: {response}\n\n\
            Critique: {critique}\n\n\
            Revision Request: {revision_request}\n\n\
            Revised Response:",
        );

        Self {
            llm,
            chain,
            principles,
            critique_prompt,
            revision_prompt,
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

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn add_principle(mut self, principle: ConstitutionalPrinciple) -> Self {
        self.principles.push(principle);
        self
    }

    pub fn default_principles() -> Vec<ConstitutionalPrinciple> {
        vec![
            ConstitutionalPrinciple::harmful_principle(),
            ConstitutionalPrinciple::offensive_principle(),
        ]
    }

    async fn critique(
        &self,
        response: &str,
        principle: &ConstitutionalPrinciple,
    ) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("principle".to_string(), principle.critique_request.clone());
        kwargs.insert("response".to_string(), response.to_string());
        let formatted = self.critique_prompt.format(&kwargs)?;

        let messages = vec![
            SystemMessage::new("You are a constitutional AI critic. Identify violations of the given principle.").into(),
            HumanMessage::new(&formatted).into(),
        ];
        let llm_response = self.llm.predict_messages(&messages, None, None).await?;
        Ok(llm_response.content)
    }

    async fn revise(
        &self,
        response: &str,
        critique: &str,
        principle: &ConstitutionalPrinciple,
    ) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("response".to_string(), response.to_string());
        kwargs.insert("critique".to_string(), critique.to_string());
        kwargs.insert("revision_request".to_string(), principle.revision_request.clone());
        let formatted = self.revision_prompt.format(&kwargs)?;

        let messages = vec![
            SystemMessage::new("You are a constitutional AI reviser. Revise the response according to the critique and revision request.").into(),
            HumanMessage::new(&formatted).into(),
        ];
        let llm_response = self.llm.predict_messages(&messages, None, None).await?;
        Ok(llm_response.content)
    }
}

#[async_trait]
impl Chain for ConstitutionalChain {
    fn input_keys(&self) -> Vec<String> {
        self.chain.input_keys()
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let chain_output = self.chain.call(inputs).await?;

        let mut current_response = chain_output
            .values()
            .next()
            .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()))
            .unwrap_or_default();

        if self.verbose {
            info!("ConstitutionalChain initial response length: {}", current_response.len());
        }

        for principle in &self.principles {
            if self.verbose {
                info!("ConstitutionalChain applying principle: {}", principle.name);
            }

            let critique = self.critique(&current_response, principle).await?;

            if self.verbose {
                info!("ConstitutionalChain critique for '{}': {}", principle.name, critique);
            }

            let critique_lower = critique.to_lowercase();
            let no_violation = critique_lower.contains("no violation")
                || critique_lower.contains("no issues")
                || critique_lower.contains("does not violate")
                || critique_lower.contains("no concerns");

            if !no_violation {
                if self.verbose {
                    info!("ConstitutionalChain revising for principle: {}", principle.name);
                }

                current_response = self
                    .revise(&current_response, &critique, principle)
                    .await?;
            }
        }

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(current_response));
        Ok(result)
    }
}
