//! Streaming endpoints — POST /{chain_name}/stream and POST /{chain_name}/stream_events.

use axum::{
    extract::Path,
    http::StatusCode,
    response::sse::{Event, Sse},
	response::Json,
    Extension, Json as JsonExtractor,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

use crate::schemas::{ErrorResponse, InvokeRequest};
use crate::server::AppState;

type SharedState = Arc<RwLock<AppState>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEventPayload {
    pub event_type: String,
    pub data: Value,
    pub run_id: Option<String>,
    pub timestamp_ms: u64,
}

pub async fn stream_chain(
    Extension(state): Extension<SharedState>,
    Path(chain_name): Path<String>,
    JsonExtractor(req): JsonExtractor<InvokeRequest>,
) -> Result<
    Sse<impl Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, Json<ErrorResponse>),
> {
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

    info!("Streaming chain '{}' with input: {:?}", chain_name, req.input);

    let input_str = req.input.to_string();
    let chars: Vec<char> = input_str.chars().collect();
    let chunk_size = (chars.len() / 4).max(1);
    let chunks: Vec<String> = chars
        .chunks(chunk_size)
        .map(|c| c.iter().collect())
        .collect();

    let stream = futures::stream::unfold(
        (0usize, chunks),
        |(i, chunks)| async move {
            if i >= chunks.len() {
                return None;
            }
            let chunk = chunks[i].clone();
            let event = Event::default().data(chunk).event("data");
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            Some((Ok::<_, Infallible>(event), (i + 1, chunks)))
        },
    );

    Ok(Sse::new(stream))
}

pub async fn stream_events(
    Extension(state): Extension<SharedState>,
    Path(chain_name): Path<String>,
    JsonExtractor(req): JsonExtractor<InvokeRequest>,
) -> Result<
    Sse<impl Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, Json<ErrorResponse>),
> {
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

    info!("Streaming events for chain '{}' with input: {:?}", chain_name, req.input);

    let run_id = uuid::Uuid::new_v4().to_string();
    let chain_name_clone = chain_name.clone();
    let input_val = req.input.clone();

    let event_types = vec![
        ("chain_start", serde_json::json!({
            "chain_name": chain_name_clone,
            "input": input_val,
            "run_id": run_id,
        })),
        ("llm_start", serde_json::json!({
            "prompt": input_val,
            "run_id": run_id,
        })),
        ("llm_new_token", serde_json::json!({
            "token": "Hello",
            "run_id": run_id,
        })),
        ("llm_new_token", serde_json::json!({
            "token": " from",
            "run_id": run_id,
        })),
        ("llm_new_token", serde_json::json!({
            "token": " LangServe",
            "run_id": run_id,
        })),
        ("llm_end", serde_json::json!({
            "output": "Hello from LangServe",
            "run_id": run_id,
        })),
        ("chain_end", serde_json::json!({
            "output": "Hello from LangServe",
            "run_id": run_id,
        })),
    ];

    let stream = futures::stream::unfold(
        (0usize, event_types),
        move |(i, events)| {
            let run_id = run_id.clone();
            async move {
                if i >= events.len() {
                    return None;
                }
                let (event_type, data) = &events[i];
                let payload = StreamEventPayload {
                    event_type: event_type.to_string(),
                    data: data.clone(),
                    run_id: Some(run_id.clone()),
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                };
                let payload_json = serde_json::to_string(&payload).unwrap_or_default();
                let event = Event::default()
                    .data(payload_json)
                    .event(*event_type);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Some((Ok::<_, Infallible>(event), (i + 1, events)))
            }
        },
    );

    Ok(Sse::new(stream))
}
