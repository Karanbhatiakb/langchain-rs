//! Google Calendar tool.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct GoogleCalendarTool {
    api_key: String,
    client: reqwest::Client,
}

impl GoogleCalendarTool {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for GoogleCalendarTool {
    fn name(&self) -> &str {
        "google_calendar"
    }

    fn description(&self) -> &str {
        "Google Calendar API tool. Supports: list_events <calendar_id>\\n<max_results>, create_event <calendar_id>\\n<summary>\\n<start_time>\\n<end_time>. Requires GOOGLE_CALENDAR_API_KEY env var. Note: Requires OAuth 2.0 for write operations."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if let Some(cmd) = input.strip_prefix("list_events ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            let calendar_id = parts[0].trim();
            let max_results = if parts.len() > 1 {
                parts[1].trim().parse::<u32>().unwrap_or(10)
            } else {
                10
            };

            let url = format!(
                "https://www.googleapis.com/calendar/v3/calendars/{}/events?maxResults={}&key={}",
                urlencode(calendar_id),
                max_results,
                self.api_key
            );
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Calendar API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Calendar API returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let items = result["items"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for item in items {
                let start = item["start"]["dateTime"]
                    .as_str()
                    .or_else(|| item["start"]["date"].as_str())
                    .unwrap_or("");
                output.push(format!(
                    "{} - {} ({})",
                    start,
                    item["summary"].as_str().unwrap_or("Untitled"),
                    item["status"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("create_event ") {
            let parts: Vec<&str> = cmd.splitn(4, '\n').collect();
            if parts.len() < 4 {
                return Err(ChainError::ToolError(
                    "Usage: create_event <calendar_id>\\n<summary>\\n<start_time>\\n<end_time>".into(),
                ));
            }
            return Err(ChainError::ToolError(
                "Google Calendar write operations require OAuth 2.0 authentication".into(),
            ));
        }

        Err(ChainError::ToolError(
            "Unknown Google Calendar command. Supported: list_events, create_event (OAuth required)".into(),
        ))
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' | '/' | '@' | ':' => c.to_string(),
            ' ' => '+'.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
