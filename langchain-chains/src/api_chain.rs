//! API chain implementation — constructs and executes API requests from natural language.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status_code: u16,
    pub body: Value,
}

pub struct APIChain {
    llm: Arc<dyn ChatModel>,
    api_request_prompt: PromptTemplate,
    api_response_prompt: PromptTemplate,
    http_client: Option<Arc<dyn HttpClient>>,
    verbose: bool,
}

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, request: ApiRequest) -> Result<ApiResponse>;
}

impl APIChain {
    pub fn new(llm: Arc<dyn ChatModel>) -> Self {
        let api_request_prompt = PromptTemplate::from_template(
            "You are given an API documentation and a question. Construct a valid API request.\n\n\
            API Docs: {api_docs}\n\n\
            Question: {input}\n\n\
            Respond with a JSON object containing:\n\
            - \"url\": the full URL\n\
            - \"method\": HTTP method (GET, POST, PUT, DELETE, PATCH)\n\
            - \"headers\": object of header key-value pairs\n\
            - \"body\": request body (if applicable, null otherwise)\n\n\
            API Request JSON:",
        );
        let api_response_prompt = PromptTemplate::from_template(
            "Given the API response below, answer the question.\n\n\
            Question: {input}\n\n\
            API Response:\n{api_response}\n\n\
            Answer:",
        );

        Self {
            llm,
            api_request_prompt,
            api_response_prompt,
            http_client: None,
            verbose: false,
        }
    }

    pub fn with_api_request_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.api_request_prompt = prompt;
        self
    }

    pub fn with_api_response_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.api_response_prompt = prompt;
        self
    }

    pub fn with_http_client(mut self, client: Arc<dyn HttpClient>) -> Self {
        self.http_client = Some(client);
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    async fn construct_api_request(
        &self,
        input: &str,
        api_docs: &str,
    ) -> Result<ApiRequest> {
        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input.to_string());
        kwargs.insert("api_docs".to_string(), api_docs.to_string());
        let prompt = self.api_request_prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let content = response.content.trim();
        let json_start = content.find('{').ok_or_else(|| {
            ChainError::ParserError("No JSON found in API request output".to_string())
        })?;
        let json_end = content.rfind('}').ok_or_else(|| {
            ChainError::ParserError("Unclosed JSON in API request output".to_string())
        })?;
        let json_str = &content[json_start..=json_end];

        let api_request: ApiRequest = serde_json::from_str(json_str)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse API request: {}", e)))?;

        Ok(api_request)
    }

    async fn execute_api_request(&self, request: ApiRequest) -> Result<ApiResponse> {
        if let Some(ref client) = self.http_client {
            client.execute(request).await
        } else {
            let body = serde_json::json!({
                "message": "No HTTP client configured",
                "url": request.url,
                "method": request.method,
            });
            Ok(ApiResponse {
                status_code: 503,
                body,
            })
        }
    }

    async fn summarize_response(
        &self,
        input: &str,
        api_response: &str,
    ) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input.to_string());
        kwargs.insert("api_response".to_string(), api_response.to_string());
        let prompt = self.api_response_prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;
        Ok(response.content)
    }
}

#[async_trait]
impl Chain for APIChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input".to_string(), "api_docs".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let api_docs = inputs
            .get("api_docs")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("APIChain constructing request for: {}", input);
        }

        let api_request = self.construct_api_request(&input, &api_docs).await?;

        if self.verbose {
            info!("APIChain executing {} {}", api_request.method, api_request.url);
        }

        let api_response = self.execute_api_request(api_request).await?;
        let response_str = serde_json::to_string(&api_response)
            .map_err(|e| ChainError::SerializationError(e.to_string()))?;

        if self.verbose {
            info!("APIChain received response with status {}", api_response.status_code);
        }

        let summary = self.summarize_response(&input, &response_str).await?;

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(summary));
        Ok(result)
    }
}
