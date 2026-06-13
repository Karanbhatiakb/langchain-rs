//! Serve crate for exposing LangChain chains, agents, and retrievers via HTTP.
//!
//! Builds on Axum to provide REST API schemas, route handlers, and middleware.

pub mod schemas;
pub mod server;
pub mod routes;
pub mod middleware;
pub mod playground;
pub mod config_routes;
pub mod batch_routes;
pub mod stream_routes;
pub mod auth;
