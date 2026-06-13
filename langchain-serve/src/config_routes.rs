//! Configuration endpoints — GET /config, GET /config/{chain_name}, POST /config/{chain_name}.

use axum::{
    extract::Path,
    http::StatusCode,
	response::Json,
    Extension, Json as JsonExtractor,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::schemas::{ChainInfo, ErrorResponse};
use crate::server::AppState;

type SharedState = Arc<RwLock<AppState>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub chains: HashMap<String, ChainInfo>,
    pub total_chains: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfigUpdate {
    pub config: Option<Value>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfigUpdateResponse {
    pub chain_name: String,
    pub updated: bool,
    pub message: String,
}

pub async fn get_config(
    Extension(state): Extension<SharedState>,
) -> Json<ConfigResponse> {
    let app_state = state.read().await;
    let chains = app_state.chains.clone();
    let total_chains = chains.len();
    Json(ConfigResponse {
        chains,
        total_chains,
    })
}

pub async fn get_chain_config(
    Extension(state): Extension<SharedState>,
    Path(chain_name): Path<String>,
) -> Result<Json<ChainInfo>, (StatusCode, Json<ErrorResponse>)> {
    let app_state = state.read().await;
    match app_state.chains.get(&chain_name) {
        Some(info) => Ok(Json(info.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Chain '{}' not found", chain_name),
                status_code: 404,
            }),
        )),
    }
}

pub async fn update_chain_config(
    Extension(state): Extension<SharedState>,
    Path(chain_name): Path<String>,
    JsonExtractor(update): JsonExtractor<ChainConfigUpdate>,
) -> Result<Json<ChainConfigUpdateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut app_state = state.write().await;
    match app_state.chains.get_mut(&chain_name) {
        Some(info) => {
            if let Some(desc) = update.description {
                info.description = Some(desc);
            }
            if let Some(tags) = update.tags {
                info.config_schema.as_object_mut().map(|obj| {
                    obj.insert(
                        "tags".to_string(),
                        Value::Array(tags.into_iter().map(Value::String).collect()),
                    );
                });
            }
            if let Some(config) = update.config {
                if let Some(obj) = config.as_object() {
                    if let Some(schema_obj) = info.config_schema.as_object_mut() {
                        for (k, v) in obj {
                            schema_obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            Ok(Json(ChainConfigUpdateResponse {
                chain_name,
                updated: true,
                message: "Configuration updated".to_string(),
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Chain '{}' not found", chain_name),
                status_code: 404,
            }),
        )),
    }
}
