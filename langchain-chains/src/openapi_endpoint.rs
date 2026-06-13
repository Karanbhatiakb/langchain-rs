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
pub struct OpenAPIRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIResponse {
    pub status_code: u16,
    pub body: Value,
}

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, request: OpenAPIRequest) -> Result<OpenAPIResponse>;
}

pub struct OpenAPIEndpointChain {
    llm: Arc<dyn ChatModel>,
    api_request_prompt: PromptTemplate,
    api_response_prompt: PromptTemplate,
    http_client: Option<Arc<dyn HttpClient>>,
    verbose: bool,
}

impl OpenAPIEndpointChain {
    pub fn new(llm: Arc<dyn ChatModel>) -> Self {
        let api_request_prompt = PromptTemplate::from_template(
            "You are given an OpenAPI spec and a question. Construct a valid API request.\n\n\
            OpenAPI Spec: {api_spec}\n\n\
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

    pub fn with_http_client(mut self, client: Arc<dyn HttpClient>) -> Self {
        self.http_client = Some(client);
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for OpenAPIEndpointChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["api_spec".to_string(), "input".to_string()]
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

        let api_spec = inputs
            .get("api_spec")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if self.verbose {
            info!("OpenAPIEndpointChain input: {}", input);
        }

        let mut request_kwargs = HashMap::new();
        request_kwargs.insert("api_spec".to_string(), api_spec);
        request_kwargs.insert("input".to_string(), input.clone());
        let request_prompt = self.api_request_prompt.format(&request_kwargs)?;

        let request_response = self
            .llm
            .predict_messages(
                &[HumanMessage::new(&request_prompt).into()],
                None,
                None,
            )
            .await?;

        let api_response = if let Some(ref client) = self.http_client {
            let parsed: OpenAPIRequest = serde_json::from_str(&request_response.content)
                .map_err(|e| ChainError::ParserError(format!("Failed to parse API request: {}", e)))?;
            client.execute(parsed).await?
        } else {
            OpenAPIResponse {
                status_code: 200,
                body: serde_json::json!({"message": "mock response"}),
            }
        };

        let api_response_str =
            serde_json::to_string(&api_response).unwrap_or_else(|_| "{}".to_string());

        let mut response_kwargs = HashMap::new();
        response_kwargs.insert("api_response".to_string(), api_response_str);
        response_kwargs.insert("input".to_string(), input);
        let response_prompt = self.api_response_prompt.format(&response_kwargs)?;

        let final_response = self
            .llm
            .predict_messages(
                &[HumanMessage::new(&response_prompt).into()],
                None,
                None,
            )
            .await?;

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(final_response.content));
        Ok(result)
    }
}
