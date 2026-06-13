//! LangSmith evaluation integration — push results, manage sessions, link evaluations to traces.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSmithConfig {
    pub api_key: String,
    pub endpoint: String,
    pub project_name: String,
}

impl LangSmithConfig {
    pub fn new(api_key: impl Into<String>, project_name: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            endpoint: "https://api.smith.langchain.com".to_string(),
            project_name: project_name.into(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSession {
    pub session_id: String,
    pub name: String,
    pub project_name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: String,
    pub reference_dataset: Option<String>,
    pub description: Option<String>,
    pub metadata: HashMap<String, Value>,
}

impl EvaluationSession {
    pub fn new(name: impl Into<String>, project_name: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            project_name: project_name.into(),
            created_at: now,
            updated_at: now,
            status: "created".to_string(),
            reference_dataset: None,
            description: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_reference_dataset(mut self, dataset: impl Into<String>) -> Self {
        self.reference_dataset = Some(dataset.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn mark_running(&mut self) {
        self.status = "running".to_string();
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    pub fn mark_completed(&mut self) {
        self.status = "completed".to_string();
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    pub fn mark_failed(&mut self) {
        self.status = "failed".to_string();
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSmithEvaluationResult {
    pub id: String,
    pub session_id: String,
    pub input: Value,
    pub prediction: Value,
    pub reference: Option<Value>,
    pub score: f64,
    pub label: Option<String>,
    pub reasoning: Option<String>,
    pub evaluator_name: String,
    pub trace_id: Option<String>,
    pub created_at: u64,
    pub metadata: HashMap<String, Value>,
}

impl LangSmithEvaluationResult {
    pub fn new(
        session_id: impl Into<String>,
        evaluator_name: impl Into<String>,
        input: Value,
        prediction: Value,
        score: f64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.into(),
            input,
            prediction,
            reference: None,
            score,
            label: None,
            reasoning: None,
            evaluator_name: evaluator_name.into(),
            trace_id: None,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            metadata: HashMap::new(),
        }
    }

    pub fn with_reference(mut self, reference: Value) -> Self {
        self.reference = Some(reference);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = Some(reasoning.into());
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSmithRun {
    pub id: String,
    pub name: String,
    pub run_type: String,
    pub inputs: Value,
    pub outputs: Option<Value>,
    pub error: Option<String>,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub trace_id: String,
    pub parent_run_id: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

impl LangSmithRun {
    pub fn new(name: impl Into<String>, run_type: impl Into<String>, inputs: Value) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let trace_id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            name: name.into(),
            run_type: run_type.into(),
            inputs,
            outputs: None,
            error: None,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            end_time: None,
            trace_id,
            parent_run_id: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_outputs(mut self, outputs: Value) -> Self {
        self.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
        self.outputs = Some(outputs);
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
        self.error = Some(error.into());
        self
    }

    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_run_id = Some(parent_id.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

pub struct LangSmithClient {
    config: LangSmithConfig,
    http_client: reqwest::Client,
    sessions: HashMap<String, EvaluationSession>,
    results: Vec<LangSmithEvaluationResult>,
    runs: HashMap<String, LangSmithRun>,
}

impl LangSmithClient {
    pub fn new(config: LangSmithConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            sessions: HashMap::new(),
            results: Vec::new(),
            runs: HashMap::new(),
        }
    }

    pub fn config(&self) -> &LangSmithConfig {
        &self.config
    }

    pub async fn create_session(
        &mut self,
        name: impl Into<String>,
    ) -> Result<EvaluationSession, Box<dyn std::error::Error + Send>> {
        let mut session = EvaluationSession::new(name, self.config.project_name.clone());
        session.mark_running();

        let url = format!("{}/api/v1/sessions", self.config.endpoint);
        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .json(&session)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Created LangSmith session: {}", session.session_id);
            }
            Ok(resp) => {
                tracing::warn!(
                    "LangSmith session creation returned status: {}, continuing locally",
                    resp.status()
                );
            }
            Err(e) => {
                tracing::warn!("Failed to reach LangSmith API: {}, continuing locally", e);
            }
        }

        let session_id = session.session_id.clone();
        self.sessions.insert(session_id.clone(), session.clone());
        Ok(session)
    }

    pub async fn update_session(
        &mut self,
        session_id: &str,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.status = status.to_string();
            session.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let url = format!("{}/api/v1/sessions/{}", self.config.endpoint, session_id);
            let _ = self
                .http_client
                .patch(&url)
                .header("x-api-key", &self.config.api_key)
                .json(&serde_json::json!({"status": status}))
                .send()
                .await;
        }
        Ok(())
    }

    pub async fn push_result(
        &mut self,
        result: LangSmithEvaluationResult,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        let url = format!(
            "{}/api/v1/sessions/{}/results",
            self.config.endpoint, result.session_id
        );
        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .json(&result)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Pushed evaluation result to LangSmith: {}", result.id);
            }
            Ok(resp) => {
                tracing::warn!(
                    "LangSmith result push returned status: {}",
                    resp.status()
                );
            }
            Err(e) => {
                tracing::warn!("Failed to push result to LangSmith: {}", e);
            }
        }

        self.results.push(result);
        Ok(())
    }

    pub async fn push_results_batch(
        &mut self,
        results: Vec<LangSmithEvaluationResult>,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        let count = results.len();
        for result in results {
            self.push_result(result).await?;
        }
        tracing::info!("Pushed {} evaluation results to LangSmith", count);
        Ok(())
    }

    pub fn create_run(
        &mut self,
        name: impl Into<String>,
        run_type: impl Into<String>,
        inputs: Value,
    ) -> LangSmithRun {
        let run = LangSmithRun::new(name, run_type, inputs);
        self.runs.insert(run.id.clone(), run.clone());
        run
    }

    pub fn end_run(
        &mut self,
        run_id: &str,
        outputs: Option<Value>,
        error: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        if let Some(run) = self.runs.get_mut(run_id) {
            run.end_time = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            );
            if let Some(out) = outputs {
                run.outputs = Some(out);
            }
            if let Some(err) = error {
                run.error = Some(err);
            }
        }
        Ok(())
    }

    pub async fn link_evaluation_to_trace(
        &mut self,
        session_id: &str,
        trace_id: &str,
        result: LangSmithEvaluationResult,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        let mut linked_result = result;
        linked_result.trace_id = Some(trace_id.to_string());

        let url = format!(
            "{}/api/v1/sessions/{}/results",
            self.config.endpoint, session_id
        );
        let _ = self
            .http_client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .json(&linked_result)
            .send()
            .await;

        self.results.push(linked_result);
        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<&EvaluationSession> {
        self.sessions.get(session_id)
    }

    pub fn get_results(&self) -> &[LangSmithEvaluationResult] {
        &self.results
    }

    pub fn get_run(&self, run_id: &str) -> Option<&LangSmithRun> {
        self.runs.get(run_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSmithSummary {
    pub session_id: String,
    pub total_results: usize,
    pub mean_score: f64,
    pub pass_rate: f64,
    pub evaluator_name: String,
}

impl LangSmithClient {
    pub fn compute_summary(&self, session_id: &str) -> Option<LangSmithSummary> {
        let session_results: Vec<&LangSmithEvaluationResult> = self
            .results
            .iter()
            .filter(|r| r.session_id == session_id)
            .collect();

        if session_results.is_empty() {
            return None;
        }

        let total = session_results.len();
        let mean_score = session_results.iter().map(|r| r.score).sum::<f64>() / total as f64;
        let pass_count = session_results.iter().filter(|r| r.score >= 0.5).count();
        let pass_rate = pass_count as f64 / total as f64;
        let evaluator_name = session_results[0].evaluator_name.clone();

        Some(LangSmithSummary {
            session_id: session_id.to_string(),
            total_results: total,
            mean_score,
            pass_rate,
            evaluator_name,
        })
    }
}
