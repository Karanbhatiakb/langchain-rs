//! Alpha Vantage financial data tool.
//!
//! Provides access to stock prices, currency exchange rates, technical
//! indicators, and other financial data via the Alpha Vantage API.

use async_trait::async_trait;
use super::traits::{BaseTool, ToolResult};

/// Tool that fetches financial data from the Alpha Vantage API.
///
/// Supports time-series data for stocks, forex, cryptocurrencies, and
/// technical indicators.
///
/// Requires the `ALPHA_VANTAGE_API_KEY` environment variable to be set.
///
/// # Stub
///
/// This is a stub implementation. Provide a valid API key and configure
/// the HTTP client to enable live data fetching.
#[derive(Debug)]
pub struct AlphaVantageTool;

impl AlphaVantageTool {
    /// Creates a new [`AlphaVantageTool`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for AlphaVantageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AlphaVantageTool {
    fn name(&self) -> &str {
        "alpha_vantage"
    }

    fn description(&self) -> &str {
        "Fetches financial data (stocks, forex, crypto, technical indicators) from the Alpha Vantage API"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from alpha_vantage (stub)".to_string())
    }
}
