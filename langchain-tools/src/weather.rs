//! Weather lookup tool.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct OpenWeatherMapTool {
    api_key: String,
}

impl OpenWeatherMapTool {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

#[async_trait]
impl BaseTool for OpenWeatherMapTool {
    fn name(&self) -> &str {
        "open_weather_map"
    }

    fn description(&self) -> &str {
        "Get the current weather for a location. Input should be a city name (e.g., 'London,UK' or 'New York'). Requires OPENWEATHER_API_KEY environment variable."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            urlencode(input.trim()),
            self.api_key
        );

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Weather API error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::ToolError(format!(
                "Weather API returned status: {}",
                resp.status()
            )));
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to parse weather data: {}", e)))?;

        let city = data["name"].as_str().unwrap_or("Unknown");
        let country = data["sys"]["country"].as_str().unwrap_or("");
        let temp = data["main"]["temp"].as_f64().unwrap_or(0.0);
        let feels_like = data["main"]["feels_like"].as_f64().unwrap_or(0.0);
        let humidity = data["main"]["humidity"].as_i64().unwrap_or(0);
        let description = data["weather"][0]["description"].as_str().unwrap_or("");
        let wind_speed = data["wind"]["speed"].as_f64().unwrap_or(0.0);

        Ok(format!(
            "Weather in {}, {}:\n\
             Temperature: {:.1}°C (feels like {:.1}°C)\n\
             Conditions: {}\n\
             Humidity: {}%\n\
             Wind Speed: {:.1} m/s",
            city, country, temp, feels_like, description, humidity, wind_speed
        ))
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' | ',' => c.to_string(),
            ' ' => '+'.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
