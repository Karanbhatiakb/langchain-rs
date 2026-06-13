//! Axum middleware for logging, auth, rate limiting, error handling, and CORS.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json},
};
use futures::FutureExt;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;
use tracing::{error, info, warn};

use crate::schemas::ErrorResponse;

/// Logs the method, URI, status code, and duration of every request.
pub async fn request_logging_middleware(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;

    let duration = start.elapsed();
    info!(
        "{} {} -> {} ({}ms)",
        method,
        uri,
        response.status(),
        duration.as_millis()
    );

    response
}

/// Validates the `x-api-key` header against an expected value.
///
/// # Errors
/// Returns `401 Unauthorized` if the key is missing or does not match.
pub async fn auth_middleware(
    req: Request,
    next: Next,
    expected_key: &str,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let auth_header = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(key) if key == expected_key => Ok(next.run(req).await),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
                message: "Invalid or missing API key".to_string(),
                status_code: 401,
            }),
        )),
    }
}

/// Sliding-window rate limiter.
pub struct RateLimiter {
    max_requests: u32,
    counter: AtomicU32,
    window_start: AtomicU64,
}

impl RateLimiter {
    /// Creates a new `RateLimiter` that allows `max_requests` per 60-second window.
    pub fn new(max_requests: u32) -> Self {
        Self {
            max_requests,
            counter: AtomicU32::new(0),
            window_start: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        }
    }

    /// Returns `true` if the request is within the rate limit.
    pub fn check(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window = self.window_start.load(Ordering::Relaxed);

        if now - window >= 60 {
            self.window_start.store(now, Ordering::Relaxed);
            self.counter.store(0, Ordering::Relaxed);
        }

        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        count < self.max_requests
    }
}

/// Enforces rate limiting per the provided `RateLimiter`.
///
/// # Errors
/// Returns `429 Too Many Requests` if the rate limit is exceeded.
pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
    limiter: &RateLimiter,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if limiter.check() {
        Ok(next.run(req).await)
    } else {
        warn!("Rate limit exceeded for {}", req.uri());
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: "rate_limited".to_string(),
                message: "Too many requests. Please try again later.".to_string(),
                status_code: 429,
            }),
        ))
    }
}

/// Catches panics in request handlers and returns a 500 response.
pub async fn error_handler_middleware(
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let result = std::panic::AssertUnwindSafe(next.run(req))
        .catch_unwind()
        .await;

    match result {
        Ok(response) => response,
        Err(_) => {
            error!("Unhandled panic in request handler");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "An internal error occurred".to_string(),
                    status_code: 500,
                }),
            )
                .into_response()
        }
    }
}

/// Adds permissive CORS headers (`*`) to every response.
pub async fn cors_middleware(req: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert(
        "access-control-allow-origin",
        "*".parse().unwrap(),
    );
    headers.insert(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "access-control-allow-headers",
        "content-type, x-api-key, authorization".parse().unwrap(),
    );

    response
}