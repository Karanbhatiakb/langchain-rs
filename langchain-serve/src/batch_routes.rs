//! Batch processing endpoints — POST /{chain_name}/batch.

use axum::{
    extract::Path,
    http::StatusCode,
	response::Json,
    Extension, Json as JsonExtractor,
};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::info;

use crate::schemas::{ErrorResponse, RunnableConfig};
use crate::server::AppState;

type SharedState = Arc<RwLock<AppState>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub output: Value,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchV2Response {
    pub outputs: Vec<BatchItemResult>,
    pub total_execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchV2Request {
    pub inputs: Vec<Value>,
    pub config: Option<RunnableConfig>,
    pub max_concurrency: Option<usize>,
}

pub async fn batch_invoke(
    Extension(state): Extension<SharedState>,
    Path(chain_name): Path<String>,
    JsonExtractor(req): JsonExtractor<BatchV2Request>,
) -> Result<Json<BatchV2Response>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();

    let chain_exists = {
        let app_state = state.read().await;
        app_state.chains.contains_key(&chain_name)
    };

    if !chain_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Chain '{}' not found", chain_name),
                status_code: 404,
            }),
        ));
    }

    let max_concurrency = req.max_concurrency.unwrap_or(8);
    info!(
        "Batch invoking chain '{}' with {} inputs (max_concurrency={})",
        chain_name,
        req.inputs.len(),
        max_concurrency
    );

    let inputs = req.inputs.clone();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrency));
    let mut handles = Vec::new();

    for input_val in inputs {
        let sem = semaphore.clone();
        let chain_name_clone = chain_name.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let item_start = Instant::now();
            info!("Processing batch item for chain '{}'", chain_name_clone);
            let execution_time_ms = item_start.elapsed().as_millis() as u64;
            BatchItemResult {
                output: input_val,
                execution_time_ms,
                metadata: HashMap::new(),
            }
        }));
    }

    let results = join_all(handles).await;
    let outputs: Vec<BatchItemResult> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let total_execution_time_ms = start.elapsed().as_millis() as u64;

    Ok(Json(BatchV2Response {
        outputs,
        total_execution_time_ms,
    }))
}
