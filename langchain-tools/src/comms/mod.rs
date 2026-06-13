//! Communication tool implementations (slack, discord, etc.).

pub use crate::slack::SlackTool;
pub use crate::discord::DiscordTool;
pub use crate::gmail::GmailTool;

use async_trait::async_trait;
use crate::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct EmailTool {
    smtp_host: String,
    smtp_port: u16,
}

impl EmailTool {
    pub fn new(smtp_host: impl Into<String>, smtp_port: u16) -> Self {
        Self {
            smtp_host: smtp_host.into(),
            smtp_port,
        }
    }
}

#[async_trait]
impl BaseTool for EmailTool {
    fn name(&self) -> &str {
        "email"
    }

    fn description(&self) -> &str {
        "Send emails via SMTP. Input format: to\\nsubject\\nbody. Requires SMTP configuration."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(3, '\n').collect();
        if parts.len() < 3 {
            return Err(ChainError::ToolError("Usage: to\\nsubject\\nbody".into()));
        }
        Err(ChainError::ToolError(
            format!("SMTP email sending not yet connected to {}:{} - use GmailTool for API-based sending", self.smtp_host, self.smtp_port),
        ))
    }
}

pub struct TwilioTool {
    account_sid: String,
    auth_token: String,
    from_number: String,
    client: reqwest::Client,
}

impl TwilioTool {
    pub fn new(account_sid: impl Into<String>, auth_token: impl Into<String>, from_number: impl Into<String>) -> Self {
        Self {
            account_sid: account_sid.into(),
            auth_token: auth_token.into(),
            from_number: from_number.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for TwilioTool {
    fn name(&self) -> &str {
        "twilio"
    }

    fn description(&self) -> &str {
        "Send SMS via Twilio. Input format: to_number\\nmessage"
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return Err(ChainError::ToolError("Usage: to_number\\nmessage".into()));
        }
        let to = parts[0].trim();
        let body = parts[1].trim();
        let url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", self.account_sid);
        let params = [("To", to), ("From", &self.from_number), ("Body", body)];
        let resp = self.client.post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send().await
            .map_err(|e| ChainError::ToolError(format!("Twilio API error: {}", e)))?;
        if resp.status().is_success() {
            Ok("SMS sent successfully".into())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(ChainError::ToolError(format!("Twilio error {}: {}", status, body)))
        }
    }
}
