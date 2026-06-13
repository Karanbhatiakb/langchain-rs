//! Twilio SMS tool for sending text messages.
//!
//! The [`TwilioTool`] sends SMS messages through the Twilio API. The input
//! should contain the recipient phone number and the message body, separated
//! by a newline.
//!
//! # Environment variables
//!
//! - `TWILIO_ACCOUNT_SID` — Twilio account SID
//! - `TWILIO_AUTH_TOKEN` — Twilio auth token
//! - `TWILIO_FROM_NUMBER` — The phone number to send from

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

/// Tool that sends an SMS message via the Twilio API.
///
/// # Input format
///
/// ```text
/// <recipient phone number>
/// <message body>
/// ```
///
/// # Stub
///
/// This is a stub implementation. Production use should replace the body of
/// [`invoke`](TwilioTool::invoke) with a real Twilio API call.
#[derive(Debug, Clone)]
pub struct TwilioTool {
    /// Twilio account SID (default: reads `TWILIO_ACCOUNT_SID` env var).
    account_sid: String,
    /// Twilio auth token (default: reads `TWILIO_AUTH_TOKEN` env var).
    auth_token: String,
    /// The phone number to send messages from (default: reads
    /// `TWILIO_FROM_NUMBER` env var).
    from_number: String,
}

impl TwilioTool {
    /// Creates a new [`TwilioTool`] reading credentials from environment
    /// variables.
    pub fn new() -> Self {
        Self {
            account_sid: std::env::var("TWILIO_ACCOUNT_SID").unwrap_or_default(),
            auth_token: std::env::var("TWILIO_AUTH_TOKEN").unwrap_or_default(),
            from_number: std::env::var("TWILIO_FROM_NUMBER").unwrap_or_default(),
        }
    }

    /// Creates a new [`TwilioTool`] with explicit credential values.
    pub fn with_credentials(
        account_sid: impl Into<String>,
        auth_token: impl Into<String>,
        from_number: impl Into<String>,
    ) -> Self {
        Self {
            account_sid: account_sid.into(),
            auth_token: auth_token.into(),
            from_number: from_number.into(),
        }
    }
}

impl Default for TwilioTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for TwilioTool {
    fn name(&self) -> &str {
        "twilio"
    }

    fn description(&self) -> &str {
        "Sends an SMS message via Twilio. \
         Input should be the recipient phone number followed by a newline \
         and the message body. Requires TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, \
         and TWILIO_FROM_NUMBER environment variables."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return Err(ChainError::ToolError(
                "Expected recipient and message body separated by a newline".into(),
            ));
        }
        let _to = parts[0].trim();
        let _body = parts[1].trim();

        if _to.is_empty() || _body.is_empty() {
            return Err(ChainError::ToolError(
                "Recipient and message body must not be empty".into(),
            ));
        }

        if self.account_sid.is_empty() || self.auth_token.is_empty() || self.from_number.is_empty() {
            return Err(ChainError::ToolError(
                "Twilio credentials not configured. Set TWILIO_ACCOUNT_SID, \
                 TWILIO_AUTH_TOKEN, and TWILIO_FROM_NUMBER environment variables."
                    .into(),
            ));
        }

        Ok("Result from twilio (stub)".to_string())
    }
}
