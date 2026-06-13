//! Server builder and runtime for LangServe (Axum-based HTTP server).

use axum::{Extension, Router};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::routes;
use crate::schemas::ChainInfo;

/// Thread-safe shared application state.
type SharedState = Arc<RwLock<AppState>>;

/// Application state holding registered chains and server start time.
pub struct AppState {
    /// Registered chain metadata, keyed by path.
    pub chains: HashMap<String, ChainInfo>,
    /// Server start timestamp used for uptime reporting.
    pub start_time: Instant,
}

impl AppState {
    /// Creates a new empty `AppState`.
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
            start_time: Instant::now(),
        }
    }
}

/// Builder and runner for the LangServe HTTP server.
pub struct LangServe {
    port: u16,
    chains: Vec<(String, ChainInfo)>,
    cors: bool,
    api_key: Option<String>,
    rate_limit: Option<u32>,
}

impl Default for LangServe {
    fn default() -> Self {
        Self::new()
    }
}

impl LangServe {
    /// Creates a new `LangServe` builder with default port 8000.
    pub fn new() -> Self {
        Self {
            port: 8000,
            chains: Vec::new(),
            cors: true,
            api_key: None,
            rate_limit: None,
        }
    }

    /// Sets the HTTP listen port.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Enables or disables CORS (default: enabled).
    pub fn with_cors(mut self, enabled: bool) -> Self {
        self.cors = enabled;
        self
    }

    /// Sets an API key required for all requests.
    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    /// Sets a rate limit (requests per minute).
    pub fn with_rate_limit(mut self, requests_per_minute: u32) -> Self {
        self.rate_limit = Some(requests_per_minute);
        self
    }

    /// Registers a chain with its metadata for serving.
    pub fn add_chain(
        &mut self,
        path: &str,
        name: &str,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
        description: Option<String>,
    ) {
        self.chains.push((
            path.to_string(),
            ChainInfo {
                name: name.to_string(),
                description,
                input_schema,
                output_schema,
                config_schema: serde_json::json!({
                    "tags": {"type": "array", "items": {"type": "string"}},
                    "recursion_limit": {"type": "integer"},
                }),
            },
        ));
    }

    /// Starts the HTTP server and begins accepting requests.
    ///
    /// # Errors
    /// Returns an error if the TCP listener fails to bind.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let state: SharedState = Arc::new(RwLock::new(AppState::new()));

        {
            let mut app_state = state.write().await;
            for (path, info) in &self.chains {
                app_state
                    .chains
                    .insert(path.clone(), info.clone());
            }
        }

        let mut app = Router::new()
            .route("/health", axum::routing::get(routes::health))
            .route("/docs", axum::routing::get(routes::docs))
            .layer(Extension(state.clone()));

        for (path, _) in &self.chains {
            let route_base = format!("/{}", path);
            app = app
                .route(
                    &format!("{}/invoke", route_base),
                    axum::routing::post(routes::invoke),
                )
                .route(
                    &format!("{}/batch", route_base),
                    axum::routing::post(routes::batch),
                )
                .route(
                    &format!("{}/stream", route_base),
                    axum::routing::post(routes::stream_handler),
                )
                .route(
                    &format!("{}/info", route_base),
                    axum::routing::get(routes::chain_info),
                );
        }

        if self.cors {
            app = app.layer(CorsLayer::permissive());
        }

        app = app.layer(TraceLayer::new_for_http());

        if let Some(ref key) = self.api_key {
            let key = key.clone();
            app = app.layer(axum::middleware::from_fn(move |req, next| {
                let key = key.clone();
                async move {
                    crate::middleware::auth_middleware(req, next, &key).await
                }
            }));
        }

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("LangServe starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }
}
