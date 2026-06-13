//! Axum route handlers for LangServe endpoints.

use axum::{
    extract::Path,
    http::StatusCode,
    response::sse::{Event, Sse},
    response::{IntoResponse, Json},
    Extension,
    Json as JsonExtractor,
};
use futures::stream::Stream;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::info;

use crate::schemas::*;
use crate::server::AppState;

type SharedState = Arc<RwLock<AppState>>;

/// Health check endpoint. Returns status, version, and uptime.
pub async fn health(Extension(state): Extension<SharedState>) -> Json<HealthResponse> {
    let uptime = state.read().await.start_time.elapsed().as_secs();
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
        uptime_seconds: uptime,
    })
}

/// Serves a Swagger UI documentation page.
pub async fn docs(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let _chains = state.read().await.chains.clone();
    let html = format!(
        r#"<!DOCTYPE html>
<html><head><title>LangServe API Docs</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
</head><body>
<div id="swagger-ui"></div>
<script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>
const ui = SwaggerUIBundle({{ url: '/openapi.json' }});
</script>
</body></html>"#
    );
    (
        StatusCode::OK,
        [("content-type", "text/html; charset=utf-8")],
        html,
    )
}

/// Returns metadata about a specific chain.
pub async fn chain_info(
    Extension(state): Extension<SharedState>,
    Path(path): Path<String>,
) -> Result<Json<ChainInfo>, (StatusCode, Json<ErrorResponse>)> {
    let state = state.read().await;
    match state.chains.get(&path) {
        Some(info) => Ok(Json(info.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Chain '{}' not found", path),
                status_code: 404,
            }),
        )),
    }
}

/// Invokes a chain with a single input.
pub async fn invoke(
    Extension(state): Extension<SharedState>,
    Path(path): Path<String>,
    JsonExtractor(req): JsonExtractor<InvokeRequest>,
) -> Result<Json<InvokeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();

    let chain_exists = {
        let state = state.read().await;
        state.chains.contains_key(&path)
    };

    if !chain_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Chain '{}' not found", path),
                status_code: 404,
            }),
        ));
    }

    info!("Invoking chain '{}' with input: {:?}", path, req.input);

    let execution_time_ms = start.elapsed().as_millis() as u64;

    Ok(Json(InvokeResponse {
        output: req.input,
        execution_time_ms,
        metadata: HashMap::new(),
    }))
}

/// Invokes a chain with a batch of inputs.
pub async fn batch(
    Extension(state): Extension<SharedState>,
    Path(path): Path<String>,
    JsonExtractor(req): JsonExtractor<BatchRequest>,
) -> Result<Json<BatchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();

    let chain_exists = {
        let state = state.read().await;
        state.chains.contains_key(&path)
    };

    if !chain_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Chain '{}' not found", path),
                status_code: 404,
            }),
        ));
    }

    info!(
        "Batch invoking chain '{}' with {} inputs",
        path,
        req.inputs.len()
    );

    let execution_time_ms = start.elapsed().as_millis() as u64;

    Ok(Json(BatchResponse {
        outputs: req.inputs,
        execution_time_ms,
    }))
}

/// Streams output from a chain via Server-Sent Events.
pub async fn stream_handler(
    Extension(state): Extension<SharedState>,
    Path(path): Path<String>,
    JsonExtractor(req): JsonExtractor<InvokeRequest>,
) -> Result<
    Sse<impl Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let chain_exists = {
        let state = state.read().await;
        state.chains.contains_key(&path)
    };

    if !chain_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Chain '{}' not found", path),
                status_code: 404,
            }),
        ));
    }

    info!("Streaming chain '{}' with input: {:?}", path, req.input);

    let chunks = vec![
        "Hello".to_string(),
        " from".to_string(),
        " LangServe".to_string(),
        "!".to_string(),
    ];

    let stream = futures::stream::unfold(
        (0usize, chunks),
        |(i, chunks)| async move {
            if i >= chunks.len() {
                return None;
            }
            let chunk = chunks[i].clone();
            let event = Event::default().data(chunk);
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            Some((Ok::<_, Infallible>(event), (i + 1, chunks)))
        },
    );

    Ok(Sse::new(stream))
}
