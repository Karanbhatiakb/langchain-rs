//! Memgraph graph database memory backend for conversation memory.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::errors::ChainError;
use langchain_core::messages::BaseMessage;
use langchain_core::messages::MessageType;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::traits::BaseMemory;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CypherQuery {
    query: String,
    params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemgraphResponse {
    results: Vec<MemgraphResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemgraphResult {
    columns: Vec<String>,
    data: Vec<MemgraphData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemgraphData {
    row: Vec<serde_json::Value>,
}

pub struct MemgraphMemory {
    client: reqwest::Client,
    base_url: String,
    session_id: String,
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    return_messages: bool,
    memory_key: String,
    input_key: String,
    output_key: String,
    human_prefix: String,
    ai_prefix: String,
}

impl MemgraphMemory {
    pub fn new(base_url: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            session_id: session_id.into(),
            chat_history: Arc::new(RwLock::new(Vec::new())),
            return_messages: false,
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            human_prefix: "Human".to_string(),
            ai_prefix: "AI".to_string(),
        }
    }

    pub fn with_return_messages(mut self, value: bool) -> Self {
        self.return_messages = value;
        self
    }

    pub fn with_memory_key(mut self, key: impl Into<String>) -> Self {
        self.memory_key = key.into();
        self
    }

    pub fn with_input_key(mut self, key: impl Into<String>) -> Self {
        self.input_key = key.into();
        self
    }

    pub fn with_output_key(mut self, key: impl Into<String>) -> Self {
        self.output_key = key.into();
        self
    }

    pub fn with_human_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.human_prefix = prefix.into();
        self
    }

    pub fn with_ai_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.ai_prefix = prefix.into();
        self
    }

    async fn execute_cypher(&self, query: &str, params: serde_json::Value) -> Result<MemgraphResponse> {
        let url = format!("{}/query", self.base_url.trim_end_matches('/'));
        let body = CypherQuery {
            query: query.to_string(),
            params,
        };
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::MemoryError(format!("Memgraph query failed: {}", e)))?;

        let response = resp
            .json::<MemgraphResponse>()
            .await
            .map_err(|e| ChainError::MemoryError(format!("Memgraph response parse failed: {}", e)))?;

        Ok(response)
    }

    pub async fn ensure_session_node(&self) -> Result<()> {
        let query = r#"
            MERGE (s:Session {id: $session_id})
        "#;
        let params = serde_json::json!({ "session_id": self.session_id });
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    pub async fn store_message(&self, role: &str, content: &str) -> Result<()> {
        self.ensure_session_node().await?;

        let query = r#"
            MATCH (s:Session {id: $session_id})
            CREATE (m:Message {role: $role, content: $content, timestamp: timestamp()})
            CREATE (s)-[:HAS_MESSAGE]->(m)
        "#;
        let params = serde_json::json!({
            "session_id": self.session_id,
            "role": role,
            "content": content
        });
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    pub async fn fetch_messages(&self) -> Result<Vec<BaseMessage>> {
        let query = r#"
            MATCH (s:Session {id: $session_id})-[:HAS_MESSAGE]->(m:Message)
            RETURN m.role AS role, m.content AS content
            ORDER BY m.timestamp ASC
        "#;
        let params = serde_json::json!({ "session_id": self.session_id });
        let response = self.execute_cypher(query, params).await?;

        let mut messages = Vec::new();
        if let Some(result) = response.results.first() {
            for data in &result.data {
                if data.row.len() >= 2 {
                    let role = data.row[0].as_str().unwrap_or("unknown");
                    let content = data.row[1].as_str().unwrap_or("");
                    let message_type = match role {
                        "human" => MessageType::Human,
                        "ai" => MessageType::AI,
                        "system" => MessageType::System,
                        "tool" => MessageType::Tool,
                        _ => MessageType::Generic,
                    };
                    messages.push(BaseMessage::new(content, message_type));
                }
            }
        }
        Ok(messages)
    }

    pub fn buffer(&self) -> String {
        let history = self.chat_history.read();
        history
            .iter()
            .map(|msg| {
                let prefix = match msg.message_type {
                    MessageType::Human => &self.human_prefix,
                    MessageType::AI => &self.ai_prefix,
                    MessageType::System => "System",
                    _ => "Unknown",
                };
                format!("{}: {}", prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl BaseMemory for MemgraphMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let messages = self.fetch_messages().await?;
        {
            let mut history = self.chat_history.write();
            *history = messages;
        }

        let mut result = HashMap::new();
        if self.return_messages {
            let msgs_value: Vec<Value> = self
                .chat_history
                .read()
                .iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            result.insert(self.memory_key.clone(), Value::Array(msgs_value));
        } else {
            result.insert(self.memory_key.clone(), Value::String(self.buffer()));
        }
        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        let input = inputs
            .get(&self.input_key)
            .or_else(|| inputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let output = outputs
            .get(&self.output_key)
            .or_else(|| outputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let input_str = input.as_str().unwrap_or("").to_string();
        let output_str = output.as_str().unwrap_or("").to_string();

        {
            let mut history = self.chat_history.write();
            history.push(BaseMessage::new(input_str.clone(), MessageType::Human));
            history.push(BaseMessage::new(output_str.clone(), MessageType::AI));
        }

        self.store_message("human", &input_str).await?;
        self.store_message("ai", &output_str).await?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let query = r#"
            MATCH (s:Session {id: $session_id})-[r:HAS_MESSAGE]->(m:Message)
            DELETE r, m
        "#;
        let params = serde_json::json!({ "session_id": self.session_id });
        self.execute_cypher(query, params).await?;

        self.chat_history.write().clear();
        Ok(())
    }
}
