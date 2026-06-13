//! Authentication middleware — API key validation (Bearer token), rate limiting (token bucket), CORS configuration.

use axum::{
    extract::Request,
	http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::warn;

use crate::schemas::ErrorResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub api_key: Option<String>,
    pub rate_limit_rpm: Option<u32>,
    pub cors_allowed_origins: Vec<String>,
    pub cors_allowed_methods: Vec<String>,
    pub cors_allowed_headers: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            rate_limit_rpm: None,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            cors_allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "x-api-key".to_string(),
            ],
        }
    }
}

impl AuthConfig {
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_rate_limit(mut self, rpm: u32) -> Self {
        self.rate_limit_rpm = Some(rpm);
        self
    }

    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.cors_allowed_origins = origins;
        self
    }
}

#[derive(Debug)]
pub struct TokenBucket {
    tokens: AtomicU64,
    max_tokens: u64,
    refill_rate: u64,
    last_refill: std::sync::Mutex<Instant>,
}

impl TokenBucket {
    pub fn new(max_tokens: u64, refill_per_second: u64) -> Self {
        Self {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
            refill_rate: refill_per_second,
            last_refill: std::sync::Mutex::new(Instant::now()),
        }
    }

    pub fn try_consume(&self) -> bool {
        self.refill();
        let current = self.tokens.load(Ordering::Relaxed);
        if current > 0 {
            self.tokens.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    fn refill(&self) {
        let mut last = self.last_refill.lock().unwrap();
        let elapsed = last.elapsed();
        if elapsed.as_secs() >= 1 {
            let tokens_to_add = (elapsed.as_secs() as u64) * self.refill_rate;
            let new_tokens = (self.tokens.load(Ordering::Relaxed) + tokens_to_add).min(self.max_tokens);
            self.tokens.store(new_tokens, Ordering::Relaxed);
            *last = Instant::now();
        }
    }
}

#[derive(Debug, Default)]
pub struct PerClientRateLimit {
    clients: RwLock<HashMap<String, Arc<TokenBucket>>>,
    max_tokens: u64,
    refill_rate: u64,
}

impl PerClientRateLimit {
    pub fn new(max_tokens: u64, refill_per_second: u64) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            max_tokens,
            refill_rate: refill_per_second,
        }
    }

    pub async fn check(&self, client_id: &str) -> bool {
        let bucket = {
            let clients = self.clients.read().await;
            clients.get(client_id).cloned()
        };

        match bucket {
            Some(b) => b.try_consume(),
            None => {
                let mut clients = self.clients.write().await;
                let new_bucket = Arc::new(TokenBucket::new(self.max_tokens, self.refill_rate));
                let allowed = new_bucket.try_consume();
                clients.insert(client_id.to_string(), new_bucket);
                allowed
            }
        }
    }
}

pub async fn bearer_auth_middleware(
    req: Request,
    next: Next,
    expected_key: &str,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let api_key_header = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    let valid = match (auth_header, api_key_header) {
        (Some(header), _) => {
            if header.starts_with("Bearer ") {
                &header[7..] == expected_key
            } else {
                header == expected_key
            }
        }
        (_, Some(key)) => key == expected_key,
        (None, None) => false,
    };

    if valid {
        Ok(next.run(req).await)
    } else {
        warn!("Bearer auth failed: invalid or missing credentials");
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
                message: "Invalid or missing Bearer token / API key".to_string(),
                status_code: 401,
            }),
        ))
    }
}

pub async fn token_bucket_rate_limit_middleware(
    req: Request,
    next: Next,
    limiter: &PerClientRateLimit,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let client_id = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string();

    if limiter.check(&client_id).await {
        Ok(next.run(req).await)
    } else {
        warn!("Rate limit exceeded for client: {}", client_id);
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: "rate_limited".to_string(),
                message: "Rate limit exceeded. Please try again later.".to_string(),
                status_code: 429,
            }),
        ))
    }
}

pub fn build_cors_layer(config: &AuthConfig) -> tower_http::cors::CorsLayer {
    let origins: Vec<_> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();

    let methods: Vec<_> = config
        .cors_allowed_methods
        .iter()
        .filter_map(|m| match m.as_str() {
            "GET" => Some(tower_http::cors::Any),
            "POST" => Some(tower_http::cors::Any),
            "PUT" => Some(tower_http::cors::Any),
            "DELETE" => Some(tower_http::cors::Any),
            "OPTIONS" => Some(tower_http::cors::Any),
            "PATCH" => Some(tower_http::cors::Any),
            _ => None,
        })
        .collect();

    let mut cors = tower_http::cors::CorsLayer::new();

    if !origins.is_empty() {
        if config.cors_allowed_origins.contains(&"*".to_string()) {
            cors = cors.allow_origin(tower_http::cors::Any);
        } else {
            cors = cors.allow_origin(origins);
        }
    }

    if !methods.is_empty() {
        cors = cors.allow_methods(tower_http::cors::Any);
    }

    let headers: Vec<_> = config
        .cors_allowed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();

    if !headers.is_empty() {
        cors = cors.allow_headers(headers);
    } else {
        cors = cors.allow_headers(tower_http::cors::Any);
    }

    cors
}
