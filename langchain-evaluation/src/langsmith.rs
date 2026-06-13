use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use langchain_core::tracers::{BaseTracer, Run};

#[derive(Debug, Serialize, Deserialize)]
pub struct LangSmithRunResponse {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LangSmithProject {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LangSmithDataset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LangSmithExample {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct LangSmithClient {
    api_key: Option<String>,
    api_url: String,
    project_name: Option<String>,
    http_client: reqwest::Client,
}

impl LangSmithClient {
    pub fn new(
        api_key: Option<String>,
        api_url: Option<String>,
        project_name: Option<String>,
    ) -> Self {
        Self {
            api_key,
            api_url: api_url.unwrap_or_else(|| "https://api.smith.langchain.com".to_string()),
            project_name,
            http_client: reqwest::Client::new(),
        }
    }

    fn is_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    fn auth_header(&self) -> Option<(String, String)> {
        self.api_key.as_ref().map(|key| ("x-api-key".to_string(), key.clone()))
    }

    pub async fn create_run(
        &self,
        run_type: &str,
        name: &str,
        inputs: Value,
        parent_run_id: Option<&str>,
        tags: Vec<String>,
        metadata: HashMap<String, Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send>> {
        if !self.is_enabled() {
            tracing::warn!("LangSmith: no API key configured, skipping create_run");
            return Ok(Uuid::new_v4().to_string());
        }

        let project = match &self.project_name {
            Some(p) => p.clone(),
            None => "default".to_string(),
        };

        let run_id = Uuid::new_v4().to_string();
        let body = serde_json::json!({
            "id": run_id,
            "name": name,
            "run_type": run_type,
            "inputs": inputs,
            "parent_run_id": parent_run_id,
            "tags": tags,
            "metadata": metadata,
            "project_name": project,
            "start_time": chrono::Utc::now().to_rfc3339(),
        });

        let url = format!("{}/api/v1/runs", self.api_url);
        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", self.api_key.as_ref().unwrap())
            .json(&body)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("LangSmith: created run {}", run_id);
            }
            Ok(resp) => {
                tracing::warn!("LangSmith: create_run returned status {}: {}", resp.status(), run_id);
            }
            Err(e) => {
                tracing::warn!("LangSmith: failed to create run: {}", e);
            }
        }

        Ok(run_id)
    }

    pub async fn update_run(
        &self,
        run_id: &str,
        outputs: Option<Value>,
        error: Option<&str>,
        status: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        if !self.is_enabled() {
            tracing::warn!("LangSmith: no API key configured, skipping update_run");
            return Ok(());
        }

        let mut body = serde_json::Map::new();
        body.insert("end_time".to_string(), Value::String(chrono::Utc::now().to_rfc3339()));
        if let Some(ref outputs) = outputs {
            body.insert("outputs".to_string(), outputs.clone());
        }
        if let Some(error) = error {
            body.insert("error".to_string(), Value::String(error.to_string()));
        }
        if let Some(status) = status {
            body.insert("status".to_string(), Value::String(status.to_string()));
        }

        let url = format!("{}/api/v1/runs/{}", self.api_url, run_id);
        let response = self
            .http_client
            .patch(&url)
            .header("x-api-key", self.api_key.as_ref().unwrap())
            .json(&Value::Object(body))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("LangSmith: updated run {}", run_id);
            }
            Ok(resp) => {
                tracing::warn!("LangSmith: update_run returned status {}: {}", resp.status(), run_id);
            }
            Err(e) => {
                tracing::warn!("LangSmith: failed to update run: {}", e);
            }
        }

        Ok(())
    }

    pub async fn create_dataset(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send>> {
        if !self.is_enabled() {
            tracing::warn!("LangSmith: no API key configured, skipping create_dataset");
            return Ok(Uuid::new_v4().to_string());
        }

        let body = serde_json::json!({
            "name": name,
            "description": description,
        });

        let url = format!("{}/api/v1/datasets", self.api_url);
        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", self.api_key.as_ref().unwrap())
            .json(&body)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<LangSmithDataset>().await {
                    Ok(dataset) => {
                        tracing::info!("LangSmith: created dataset {} ({})", name, dataset.id);
                        Ok(dataset.id)
                    }
                    Err(e) => {
                        tracing::warn!("LangSmith: failed to parse dataset response: {}", e);
                        Ok(Uuid::new_v4().to_string())
                    }
                }
            }
            Ok(resp) => {
                let status = resp.status();
                tracing::warn!("LangSmith: create_dataset returned status {}", status);
                Ok(Uuid::new_v4().to_string())
            }
            Err(e) => {
                tracing::warn!("LangSmith: failed to create dataset: {}", e);
                Ok(Uuid::new_v4().to_string())
            }
        }
    }

    pub async fn create_example(
        &self,
        dataset_id: &str,
        inputs: Value,
        outputs: Option<Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send>> {
        if !self.is_enabled() {
            tracing::warn!("LangSmith: no API key configured, skipping create_example");
            return Ok(Uuid::new_v4().to_string());
        }

        let body = serde_json::json!({
            "dataset_id": dataset_id,
            "inputs": inputs,
            "outputs": outputs,
        });

        let url = format!("{}/api/v1/examples", self.api_url);
        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", self.api_key.as_ref().unwrap())
            .json(&body)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<LangSmithExample>().await {
                    Ok(example) => {
                        tracing::info!("LangSmith: created example {}", example.id);
                        Ok(example.id)
                    }
                    Err(e) => {
                        tracing::warn!("LangSmith: failed to parse example response: {}", e);
                        Ok(Uuid::new_v4().to_string())
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!("LangSmith: create_example returned status {}", resp.status());
                Ok(Uuid::new_v4().to_string())
            }
            Err(e) => {
                tracing::warn!("LangSmith: failed to create example: {}", e);
                Ok(Uuid::new_v4().to_string())
            }
        }
    }

    pub async fn list_projects(
        &self,
    ) -> Result<Vec<LangSmithProject>, Box<dyn std::error::Error + Send>> {
        if !self.is_enabled() {
            tracing::warn!("LangSmith: no API key configured, skipping list_projects");
            return Ok(Vec::new());
        }

        let url = format!("{}/api/v1/projects", self.api_url);
        let response = self
            .http_client
            .get(&url)
            .header("x-api-key", self.api_key.as_ref().unwrap())
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Vec<LangSmithProject>>().await {
                    Ok(projects) => Ok(projects),
                    Err(e) => {
                        tracing::warn!("LangSmith: failed to parse projects response: {}", e);
                        Ok(Vec::new())
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!("LangSmith: list_projects returned status {}", resp.status());
                Ok(Vec::new())
            }
            Err(e) => {
                tracing::warn!("LangSmith: failed to list projects: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

pub struct LangSmithTracer {
    client: LangSmithClient,
}

impl LangSmithTracer {
    pub fn new(client: LangSmithClient) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &LangSmithClient {
        &self.client
    }
}

#[async_trait]
impl BaseTracer for LangSmithTracer {
    async fn on_run_create(&self, run: &Run) {
        let run_type_str = run.run_type.to_string();
        let tags = run.tags.clone();
        let metadata = match &run.metadata {
            Value::Object(m) => m.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            _ => HashMap::new(),
        };
        let parent_run_id = run.parent_run_id.map(|id| id.to_string());
        let _ = self
            .client
            .create_run(
                &run_type_str,
                &run.name,
                run.inputs.clone(),
                parent_run_id.as_deref(),
                tags,
                metadata,
            )
            .await;
    }

    async fn on_run_update(&self, run: &Run) {
        let status = Some(match run.status {
            langchain_core::tracers::RunStatus::Running => "running",
            langchain_core::tracers::RunStatus::Succeeded => "succeeded",
            langchain_core::tracers::RunStatus::Failed => "failed",
            langchain_core::tracers::RunStatus::Cancelled => "cancelled",
            langchain_core::tracers::RunStatus::NotStarted => "not_started",
        });
        let _ = self
            .client
            .update_run(
                &run.id.to_string(),
                run.outputs.clone(),
                run.error.as_deref(),
                status,
            )
            .await;
    }

    async fn on_run_end(&self, run: &Run) {
        let _ = self
            .client
            .update_run(
                &run.id.to_string(),
                run.outputs.clone(),
                None,
                Some("succeeded"),
            )
            .await;
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        let _ = self
            .client
            .update_run(
                &run.id.to_string(),
                None,
                Some(error),
                Some("failed"),
            )
            .await;
    }

    fn get_runs(&self) -> Vec<Run> {
        Vec::new()
    }

    fn get_run(&self, _run_id: Uuid) -> Option<Run> {
        None
    }
}

pub struct LangSmithEvaluator {
    client: LangSmithClient,
}

impl LangSmithEvaluator {
    pub fn new(client: LangSmithClient) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &LangSmithClient {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use langchain_core::tracers::RunType;

    #[tokio::test]
    async fn test_client_new_without_api_key() {
        let client = LangSmithClient::new(None, None, None);
        assert!(client.api_key.is_none());
        assert_eq!(client.api_url, "https://api.smith.langchain.com");
    }

    #[tokio::test]
    async fn test_client_create_run_no_api_key() {
        let client = LangSmithClient::new(None, None, None);
        let run_id = client
            .create_run("llm", "test", Value::Null, None, vec![], HashMap::new())
            .await
            .unwrap();
        assert!(!run_id.is_empty());
    }

    #[tokio::test]
    async fn test_client_update_run_no_api_key() {
        let client = LangSmithClient::new(None, None, None);
        let result = client
            .update_run("test-id", Some(Value::Null), None, Some("succeeded"))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_create_dataset_no_api_key() {
        let client = LangSmithClient::new(None, None, None);
        let dataset_id = client.create_dataset("test-dataset", None).await.unwrap();
        assert!(!dataset_id.is_empty());
    }

    #[tokio::test]
    async fn test_client_create_example_no_api_key() {
        let client = LangSmithClient::new(None, None, None);
        let example_id = client
            .create_example("ds-1", serde_json::json!({"input": "hello"}), None)
            .await
            .unwrap();
        assert!(!example_id.is_empty());
    }

    #[tokio::test]
    async fn test_client_list_projects_no_api_key() {
        let client = LangSmithClient::new(None, None, None);
        let projects = client.list_projects().await.unwrap();
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn test_tracer_create() {
        let client = LangSmithClient::new(None, None, None);
        let tracer = LangSmithTracer::new(client);
        let run = Run::new(
            Uuid::new_v4(),
            "test",
            RunType::Llm,
            None,
            Value::Null,
        );
        tracer.on_run_create(&run).await;
        assert!(tracer.get_runs().is_empty());
    }

    #[tokio::test]
    async fn test_tracer_on_run_end() {
        let client = LangSmithClient::new(None, None, None);
        let tracer = LangSmithTracer::new(client);
        let mut run = Run::new(
            Uuid::new_v4(),
            "test",
            RunType::Chain,
            None,
            Value::Null,
        );
        run.succeed(serde_json::json!({"result": "ok"}));
        tracer.on_run_end(&run).await;
    }

    #[tokio::test]
    async fn test_evaluator_create() {
        let client = LangSmithClient::new(None, None, None);
        let evaluator = LangSmithEvaluator::new(client);
        assert!(!evaluator.client().is_enabled());
    }
}
