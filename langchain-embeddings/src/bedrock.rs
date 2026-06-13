//! AWS Bedrock embedding model provider.

use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{sign, SigningSettings, SigningParams, SignableRequest};
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::SystemTime;
use url::Url;

use crate::traits::Embeddings;

pub struct BedrockEmbeddings {
    model_id: String,
    region: String,
    client: Client,
    access_key: String,
    secret_key: String,
    session_token: Option<String>,
}

impl std::fmt::Debug for BedrockEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BedrockEmbeddings")
            .field("model_id", &self.model_id)
            .field("region", &self.region)
            .finish()
    }
}

impl BedrockEmbeddings {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            region: std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            client: Client::new(),
            access_key: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
        }
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    pub fn with_credentials(mut self, access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        self.access_key = access_key.into();
        self.secret_key = secret_key.into();
        self
    }

    pub fn with_session_token(mut self, token: impl Into<String>) -> Self {
        self.session_token = Some(token.into());
        self
    }

    fn credentials(&self) -> Credentials {
        Credentials::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            self.session_token.clone(),
            None,
            "bedrock",
        )
    }

    async fn invoke_model(&self, body: &[u8]) -> Result<Value> {
        let url = format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            self.region, self.model_id
        );
        let creds = self.credentials();

        let parsed_url =
            Url::parse(&url).map_err(|e| ChainError::LLMError(format!("Invalid URL: {}", e)))?;
        let host = parsed_url.host_str().unwrap_or("").to_string();

        let http_req = http::Request::builder()
            .method("POST")
            .uri(&url)
            .header("Content-Type", "application/json")
            .header("Host", &host)
            .body(Vec::from(body))
            .map_err(|e| ChainError::LLMError(format!("Failed to build request: {}", e)))?;

        let signable = SignableRequest::from(&http_req);

        let signing_params = SigningParams::builder()
            .access_key(creds.access_key_id())
            .secret_key(creds.secret_access_key())
            .security_token(creds.session_token().map(|s| s.to_string()))
            .region(&self.region)
            .service_name("bedrock")
            .time(SystemTime::now())
            .settings(SigningSettings::default())
            .build()
            .map_err(|e| ChainError::LLMError(format!("Failed to build signing params: {}", e)))?;

        let (signed_req, _) = sign(signable, &signing_params.into())
            .map_err(|e| ChainError::LLMError(format!("Failed to sign request: {}", e)))?;

        let signed_headers = signed_req.headers();
        let mut reqwest_headers = reqwest::header::HeaderMap::new();
        for (name, value) in signed_headers {
            if let Ok(h_name) = reqwest::header::HeaderName::from_bytes(name.as_ref()) {
                if let Ok(h_value) = reqwest::header::HeaderValue::from_bytes(value.as_ref()) {
                    reqwest_headers.insert(h_name, h_value);
                }
            }
        }

        let response = self
            .client
            .post(&url)
            .headers(reqwest_headers)
            .body(body.to_vec())
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Bedrock request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            return Err(ChainError::EmbeddingError(format!(
                "Bedrock invoke error ({}): {}",
                status, err_text
            )));
        }

        let resp: Value = response
            .json()
            .await
            .map_err(|e| ChainError::EmbeddingError(format!("Failed to parse response: {}", e)))?;

        Ok(resp)
    }
}

#[async_trait]
impl Embeddings for BedrockEmbeddings {
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            let body = serde_json::json!({ "inputText": text }).to_string().into_bytes();
            let resp = self.invoke_model(&body).await?;
            let emb: Vec<f32> = resp["embedding"]
                .as_array()
                .ok_or_else(|| ChainError::EmbeddingError("Missing embedding in response".to_string()))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            embeddings.push(emb);
        }
        Ok(embeddings)
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let body = serde_json::json!({ "inputText": text }).to_string().into_bytes();
        let resp = self.invoke_model(&body).await?;
        let emb: Vec<f32> = resp["embedding"]
            .as_array()
            .ok_or_else(|| ChainError::EmbeddingError("Missing embedding in response".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        Ok(emb)
    }

    fn embedding_dimension(&self) -> usize {
        if self.model_id.starts_with("amazon.titan-embed-text-v2") {
            1024
        } else if self.model_id.starts_with("amazon.titan-embed-text") {
            1536
        } else if self.model_id.starts_with("cohere") {
            4096
        } else if self.model_id.starts_with("ai21") {
            2048
        } else {
            1536
        }
    }
}
