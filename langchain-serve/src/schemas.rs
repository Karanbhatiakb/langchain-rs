//! Request/response schemas for the LangServe HTTP API.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Request body for invoking a runnable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeRequest {
    /// The input value to pass to the runnable.
    pub input: Value,
    /// Optional runnable configuration.
    pub config: Option<RunnableConfig>,
}

/// Configuration for runnable execution (API version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnableConfig {
    /// Tags for filtering and tracing.
    pub tags: Option<Vec<String>>,
    /// Arbitrary metadata.
    pub metadata: Option<HashMap<String, Value>>,
    /// Callback handler names.
    pub callbacks: Option<Vec<String>>,
    /// Maximum recursion depth.
    pub recursion_limit: Option<u32>,
    /// Maximum concurrent operations.
    pub max_concurrency: Option<u32>,
}

/// Response from a runnable invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeResponse {
    /// The output value from the runnable.
    pub output: Value,
    /// Wall-clock execution time in milliseconds.
    pub execution_time_ms: u64,
    /// Additional response metadata.
    pub metadata: HashMap<String, Value>,
}

/// Request body for batch invocations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// List of input values.
    pub inputs: Vec<Value>,
    /// Optional runnable configuration.
    pub config: Option<RunnableConfig>,
}

/// Response from a batch invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// The output values in the same order as the inputs.
    pub outputs: Vec<Value>,
    /// Wall-clock execution time in milliseconds.
    pub execution_time_ms: u64,
}

/// A streaming event emitted by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// The event type (e.g., "data", "error", "end").
    pub event: String,
    /// The event payload.
    pub data: Value,
}

/// Metadata about a registered chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    /// The chain's name.
    pub name: String,
    /// An optional description.
    pub description: Option<String>,
    /// JSON Schema for the expected input.
    pub input_schema: Value,
    /// JSON Schema for the produced output.
    pub output_schema: Value,
    /// JSON Schema for the runnable configuration.
    pub config_schema: Value,
}

/// Response from the health endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Current status (e.g., "ok").
    pub status: String,
    /// Server version.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
}

/// Standard error response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// The error type identifier.
    pub error: String,
    /// A human-readable error message.
    pub message: String,
    /// The HTTP status code.
    pub status_code: u16,
}
