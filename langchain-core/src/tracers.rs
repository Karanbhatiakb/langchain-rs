//! Tracing infrastructure for LangChain run execution.
//!
//! Provides types for tracking the lifecycle of runnable executions ([`Run`]),
//! the [`BaseTracer`] trait for implementing custom tracers, and several
//! built-in tracer implementations:
//!
//! - [`LangChainTracer`] — tracks runs in memory, designed for LangSmith integration
//! - [`RunCollector`] — collects all runs into a `Vec`
//! - [`LoggingTracer`] — logs run events via the `tracing` crate
//! - [`EvaluationTracer`] — tracer for evaluation runs
//! - [`StdoutTracer`] — prints run info to stdout
//! - [`LogStreamTracer`] — streams logs as structured JSON
//!
//! Context management is available via [`TracerContext`], and root-level
//! listener patterns via [`RootListeners`].

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

/// Classification of the kind of runnable a [`Run`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    /// A large-language-model invocation.
    Llm,
    /// A chat-model invocation.
    ChatModel,
    /// A chain of operations.
    Chain,
    /// A tool invocation.
    Tool,
    /// A retriever invocation.
    Retriever,
    /// An agent planning step.
    Agent,
    /// A memory operation.
    Memory,
    /// A callback handler invocation.
    Callback,
    /// A prompt construction step.
    Prompt,
    /// Any other or custom run type.
    Other,
}

impl fmt::Display for RunType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunType::Llm => write!(f, "llm"),
            RunType::ChatModel => write!(f, "chat_model"),
            RunType::Chain => write!(f, "chain"),
            RunType::Tool => write!(f, "tool"),
            RunType::Retriever => write!(f, "retriever"),
            RunType::Agent => write!(f, "agent"),
            RunType::Memory => write!(f, "memory"),
            RunType::Callback => write!(f, "callback"),
            RunType::Prompt => write!(f, "prompt"),
            RunType::Other => write!(f, "other"),
        }
    }
}

/// Current status of a [`Run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// The run has been created but has not started executing.
    NotStarted,
    /// The run is currently executing.
    Running,
    /// The run completed successfully.
    Succeeded,
    /// The run failed with an error.
    Failed,
    /// The run was cancelled before completion.
    Cancelled,
}

impl fmt::Display for RunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunStatus::NotStarted => write!(f, "not_started"),
            RunStatus::Running => write!(f, "running"),
            RunStatus::Succeeded => write!(f, "succeeded"),
            RunStatus::Failed => write!(f, "failed"),
            RunStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// A named event recorded during a [`Run`].
///
/// Events are appended over the lifetime of a run (e.g. "start", "new_token",
/// "end", "error") and carry an optional JSON data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event name (e.g. `"start"`, `"end"`, `"error"`, `"new_token"`).
    pub name: String,
    /// Timestamp when the event occurred.
    pub time: DateTime<Utc>,
    /// Optional JSON data associated with the event.
    pub data: Option<Value>,
}

/// Tracks the full lifecycle of a single runnable execution.
///
/// A `Run` is the central data structure in the tracing subsystem. It records
/// the identity, timing, inputs, outputs, errors, tags, metadata, and events
/// for one invocation of an LLM, chain, tool, retriever, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    /// Unique identifier for this run.
    pub id: Uuid,
    /// Human-readable name of the runnable.
    pub name: String,
    /// The kind of runnable this run represents.
    pub run_type: RunType,
    /// ID of the parent run, if this is a sub-run.
    pub parent_run_id: Option<Uuid>,
    /// Timestamp when the run started.
    pub start_time: DateTime<Utc>,
    /// Timestamp when the run ended, if it has finished.
    pub end_time: Option<DateTime<Utc>>,
    /// JSON representation of the run inputs.
    pub inputs: Value,
    /// JSON representation of the run outputs, if available.
    pub outputs: Option<Value>,
    /// Error message if the run failed.
    pub error: Option<String>,
    /// Current status of the run.
    pub status: RunStatus,
    /// Tags attached to the run.
    pub tags: Vec<String>,
    /// Arbitrary metadata associated with the run.
    pub metadata: Value,
    /// Ordered list of events recorded during the run.
    pub events: Vec<Event>,
    /// Child runs spawned by this run.
    pub child_runs: Vec<Run>,
}

impl Run {
    /// Creates a new `Run` in the [`Running`](RunStatus::Running) state.
    ///
    /// The `start_time` is set to the current UTC time and an initial
    /// `"start"` event is appended.
    pub fn new(
        id: Uuid,
        name: impl Into<String>,
        run_type: RunType,
        parent_run_id: Option<Uuid>,
        inputs: Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name: name.into(),
            run_type,
            parent_run_id,
            start_time: now,
            end_time: None,
            inputs,
            outputs: None,
            error: None,
            status: RunStatus::Running,
            tags: Vec::new(),
            metadata: Value::Object(serde_json::Map::new()),
            events: vec![Event {
                name: "start".to_string(),
                time: now,
                data: None,
            }],
            child_runs: Vec::new(),
        }
    }

    /// Marks the run as successfully completed with the given outputs.
    pub fn succeed(&mut self, outputs: Value) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.outputs = Some(outputs);
        self.status = RunStatus::Succeeded;
        self.events.push(Event {
            name: "end".to_string(),
            time: now,
            data: None,
        });
    }

    /// Marks the run as failed with the given error message.
    pub fn fail(&mut self, error: impl Into<String>) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.error = Some(error.into());
        self.status = RunStatus::Failed;
        self.events.push(Event {
            name: "error".to_string(),
            time: now,
            data: None,
        });
    }

    /// Marks the run as cancelled.
    pub fn cancel(&mut self) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.status = RunStatus::Cancelled;
        self.events.push(Event {
            name: "cancel".to_string(),
            time: now,
            data: None,
        });
    }

    /// Computes the elapsed duration of the run, if it has ended.
    pub fn elapsed(&self) -> Option<chrono::Duration> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Appends a child run to this run.
    pub fn add_child_run(&mut self, child: Run) {
        self.child_runs.push(child);
    }
}

// ---------------------------------------------------------------------------
// BaseTracer trait
// ---------------------------------------------------------------------------

/// Core trait for all tracers.
///
/// A tracer receives notifications as runs are created, updated, completed,
/// or encounter errors. Implementations may persist runs, log them, collect
/// them for later inspection, or stream them to external consumers.
///
/// All implementors must be `Send + Sync + 'static` so they can be shared
/// across async tasks and threads.
#[async_trait]
pub trait BaseTracer: Send + Sync + 'static {
    /// Called when a new run is created and starts executing.
    async fn on_run_create(&self, run: &Run);

    /// Called when a run is updated (e.g. intermediate results, new events).
    async fn on_run_update(&self, run: &Run);

    /// Called when a run completes successfully.
    async fn on_run_end(&self, run: &Run);

    /// Called when a run encounters an error.
    async fn on_run_error(&self, run: &Run, error: &str);

    /// Returns all runs known to this tracer.
    fn get_runs(&self) -> Vec<Run>;

    /// Returns a specific run by its ID, if known.
    fn get_run(&self, run_id: Uuid) -> Option<Run>;
}

// ---------------------------------------------------------------------------
// LangChainTracer
// ---------------------------------------------------------------------------

/// In-memory tracer that tracks runs in an `Arc<RwLock<HashMap>>`.
///
/// Designed as the Rust analogue of the Python `LangChainTracer` that ships
/// runs to LangSmith. This implementation keeps all runs in memory and
/// provides a [`LangChainTracer::flush`] method for downstream consumers
/// (e.g. a future LangSmith client) to drain the stored runs.
#[derive(Debug)]
pub struct LangChainTracer {
    runs: Arc<RwLock<HashMap<Uuid, Run>>>,
    project_name: Option<String>,
    tags: Vec<String>,
    latest_run_id: Arc<RwLock<Option<Uuid>>>,
}

impl LangChainTracer {
    /// Creates a new `LangChainTracer` with an optional project name and tags.
    pub fn new(project_name: Option<String>, tags: Vec<String>) -> Self {
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
            project_name,
            tags,
            latest_run_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the project name, if set.
    pub fn project_name(&self) -> Option<&str> {
        self.project_name.as_deref()
    }

    /// Returns the tags configured on this tracer.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Drains and returns all persisted runs, clearing the internal map.
    pub async fn flush(&self) -> Vec<Run> {
        let mut runs = self.runs.write().await;
        let drained: Vec<Run> = runs.drain().map(|(_, v)| v).collect();
        drained
    }

    /// Returns the ID of the most recently persisted root run.
    pub async fn latest_run_id(&self) -> Option<Uuid> {
        let guard = self.latest_run_id.read().await;
        *guard
    }
}

#[async_trait]
impl BaseTracer for LangChainTracer {
    async fn on_run_create(&self, run: &Run) {
        let mut runs = self.runs.write().await;
        runs.insert(run.id, run.clone());
        if run.parent_run_id.is_none() {
            let mut latest = self.latest_run_id.write().await;
            *latest = Some(run.id);
        }
    }

    async fn on_run_update(&self, run: &Run) {
        let mut runs = self.runs.write().await;
        runs.insert(run.id, run.clone());
    }

    async fn on_run_end(&self, run: &Run) {
        let mut runs = self.runs.write().await;
        runs.insert(run.id, run.clone());
        if run.parent_run_id.is_none() {
            let mut latest = self.latest_run_id.write().await;
            *latest = Some(run.id);
        }
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        let mut failed = run.clone();
        failed.fail(error);
        let mut runs = self.runs.write().await;
        runs.insert(failed.id, failed);
    }

    fn get_runs(&self) -> Vec<Run> {
        tokio::task::block_in_place(|| {
            self.runs.blocking_read().values().cloned().collect()
        })
    }

    fn get_run(&self, run_id: Uuid) -> Option<Run> {
        tokio::task::block_in_place(|| {
            let runs = self.runs.blocking_read();
            runs.get(&run_id).cloned()
        })
    }
}

impl Clone for LangChainTracer {
    fn clone(&self) -> Self {
        Self {
            runs: Arc::clone(&self.runs),
            project_name: self.project_name.clone(),
            tags: self.tags.clone(),
            latest_run_id: Arc::clone(&self.latest_run_id),
        }
    }
}

// ---------------------------------------------------------------------------
// RunCollector
// ---------------------------------------------------------------------------

/// Simple tracer that collects all runs into a `Vec<Run>`.
///
/// Useful for inspection and evaluation purposes. This is the Rust analogue
/// of the Python `RunCollectorCallbackHandler`.
#[derive(Debug)]
pub struct RunCollector {
    runs: Arc<RwLock<Vec<Run>>>,
    example_id: Option<Uuid>,
}

impl RunCollector {
    /// Creates a new `RunCollector` with an optional example ID.
    pub fn new(example_id: Option<Uuid>) -> Self {
        Self {
            runs: Arc::new(RwLock::new(Vec::new())),
            example_id,
        }
    }

    /// Returns the example ID, if set.
    pub fn example_id(&self) -> Option<Uuid> {
        self.example_id
    }

    /// Returns the number of collected runs.
    pub async fn len(&self) -> usize {
        self.runs.read().await.len()
    }

    /// Returns `true` if no runs have been collected.
    pub async fn is_empty(&self) -> bool {
        self.runs.read().await.is_empty()
    }

    /// Drains and returns all collected runs, clearing the internal list.
    pub async fn drain(&self) -> Vec<Run> {
        self.runs.write().await.drain(..).collect()
    }
}

#[async_trait]
impl BaseTracer for RunCollector {
    async fn on_run_create(&self, run: &Run) {
        self.runs.write().await.push(run.clone());
    }

    async fn on_run_update(&self, run: &Run) {
        let mut runs = self.runs.write().await;
        if let Some(existing) = runs.iter_mut().find(|r| r.id == run.id) {
            *existing = run.clone();
        } else {
            runs.push(run.clone());
        }
    }

    async fn on_run_end(&self, run: &Run) {
        let mut runs = self.runs.write().await;
        if let Some(existing) = runs.iter_mut().find(|r| r.id == run.id) {
            *existing = run.clone();
        } else {
            runs.push(run.clone());
        }
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        let mut failed = run.clone();
        failed.fail(error);
        let mut runs = self.runs.write().await;
        if let Some(existing) = runs.iter_mut().find(|r| r.id == failed.id) {
            *existing = failed;
        } else {
            runs.push(failed);
        }
    }

    fn get_runs(&self) -> Vec<Run> {
        tokio::task::block_in_place(|| {
            self.runs.blocking_read().clone()
        })
    }

    fn get_run(&self, run_id: Uuid) -> Option<Run> {
        tokio::task::block_in_place(|| {
            self.runs.blocking_read().iter().find(|r| r.id == run_id).cloned()
        })
    }
}

impl Clone for RunCollector {
    fn clone(&self) -> Self {
        Self {
            runs: Arc::clone(&self.runs),
            example_id: self.example_id,
        }
    }
}

// ---------------------------------------------------------------------------
// LoggingTracer
// ---------------------------------------------------------------------------

/// Tracer that emits structured log events via the `tracing` crate.
///
/// Each lifecycle event is logged at an appropriate level:
/// - run creation and completion at `INFO`
/// - errors at `ERROR`
/// - updates at `DEBUG`
#[derive(Debug)]
pub struct LoggingTracer;

impl LoggingTracer {
    /// Creates a new `LoggingTracer`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoggingTracer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTracer for LoggingTracer {
    async fn on_run_create(&self, run: &Run) {
        tracing::info!(
            run_id = %run.id,
            name = %run.name,
            run_type = %run.run_type,
            parent_run_id = ?run.parent_run_id,
            "Run created"
        );
    }

    async fn on_run_update(&self, run: &Run) {
        tracing::debug!(
            run_id = %run.id,
            name = %run.name,
            status = %run.status,
            "Run updated"
        );
    }

    async fn on_run_end(&self, run: &Run) {
        let elapsed = run
            .end_time
            .map(|t| (t - run.start_time).to_string())
            .unwrap_or_else(|| "unknown".to_string());
        tracing::info!(
            run_id = %run.id,
            name = %run.name,
            run_type = %run.run_type,
            elapsed = %elapsed,
            "Run ended"
        );
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        tracing::error!(
            run_id = %run.id,
            name = %run.name,
            run_type = %run.run_type,
            error = %error,
            "Run error"
        );
    }

    fn get_runs(&self) -> Vec<Run> {
        Vec::new()
    }

    fn get_run(&self, _run_id: Uuid) -> Option<Run> {
        None
    }
}

impl Clone for LoggingTracer {
    fn clone(&self) -> Self {
        Self
    }
}

// ---------------------------------------------------------------------------
// EvaluationTracer
// ---------------------------------------------------------------------------

/// Tracer for evaluation runs.
///
/// Collects completed root runs (those without a `parent_run_id`) so that
/// evaluation logic can inspect them after execution. Runs that have not
/// produced outputs are optionally skipped.
#[derive(Debug)]
pub struct EvaluationTracer {
    runs: Arc<RwLock<Vec<Run>>>,
    skip_unfinished: bool,
}

impl EvaluationTracer {
    /// Creates a new `EvaluationTracer`.
    ///
    /// When `skip_unfinished` is `true`, runs without outputs are not
    /// persisted into the evaluation collection.
    pub fn new(skip_unfinished: bool) -> Self {
        Self {
            runs: Arc::new(RwLock::new(Vec::new())),
            skip_unfinished,
        }
    }

    /// Returns whether unfinished runs are skipped.
    pub fn skip_unfinished(&self) -> bool {
        self.skip_unfinished
    }

    /// Returns the collected evaluation runs.
    pub async fn evaluation_runs(&self) -> Vec<Run> {
        self.runs.read().await.clone()
    }

    /// Returns the number of collected evaluation runs.
    pub async fn evaluation_run_count(&self) -> usize {
        self.runs.read().await.len()
    }
}

#[async_trait]
impl BaseTracer for EvaluationTracer {
    async fn on_run_create(&self, _run: &Run) {}

    async fn on_run_update(&self, _run: &Run) {}

    async fn on_run_end(&self, run: &Run) {
        if run.parent_run_id.is_some() {
            return;
        }
        if self.skip_unfinished && run.outputs.is_none() {
            tracing::debug!(run_id = %run.id, "Skipping unfinished run in evaluation tracer");
            return;
        }
        self.runs.write().await.push(run.clone());
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        if run.parent_run_id.is_some() {
            return;
        }
        let mut failed = run.clone();
        failed.fail(error);
        self.runs.write().await.push(failed);
    }

    fn get_runs(&self) -> Vec<Run> {
        tokio::task::block_in_place(|| {
            self.runs.blocking_read().clone()
        })
    }

    fn get_run(&self, run_id: Uuid) -> Option<Run> {
        tokio::task::block_in_place(|| {
            self.runs.blocking_read().iter().find(|r| r.id == run_id).cloned()
        })
    }
}

impl Clone for EvaluationTracer {
    fn clone(&self) -> Self {
        Self {
            runs: Arc::clone(&self.runs),
            skip_unfinished: self.skip_unfinished,
        }
    }
}

// ---------------------------------------------------------------------------
// StdoutTracer
// ---------------------------------------------------------------------------

/// Tracer that prints run information to stdout.
///
/// This is the Rust analogue of the Python `ConsoleCallbackHandler`.
/// Output is formatted with run type, name, breadcrumbs, and timing.
#[derive(Debug)]
pub struct StdoutTracer;

impl StdoutTracer {
    /// Creates a new `StdoutTracer`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdoutTracer {
    fn default() -> Self {
        Self::new()
    }
}

fn format_elapsed(run: &Run) -> String {
    match run.elapsed() {
        Some(d) => {
            let ms = d.num_milliseconds();
            if ms < 1000 {
                format!("{}ms", ms)
            } else {
                format!("{:.2}s", ms as f64 / 1000.0)
            }
        }
        None => "unknown".to_string(),
    }
}

#[async_trait]
impl BaseTracer for StdoutTracer {
    async fn on_run_create(&self, run: &Run) {
        let run_type = run.run_type.to_string();
        let run_type_upper = {
            let mut s = run_type;
            let first = s.chars().next().map(|c| c.to_ascii_uppercase());
            if let Some(c) = first {
                s.replace_range(..c.len_utf8(), &c.to_string());
            }
            s
        };
        println!(
            "[{}/start] [{}] Entering {} run with input:\n{}",
            run.run_type,
            run.name,
            run_type_upper,
            serde_json::to_string_pretty(&run.inputs)
                .unwrap_or_else(|_| "[inputs]".to_string())
        );
    }

    async fn on_run_update(&self, _run: &Run) {}

    async fn on_run_end(&self, run: &Run) {
        let outputs_str = match &run.outputs {
            Some(o) => serde_json::to_string_pretty(o).unwrap_or_else(|_| "[outputs]".to_string()),
            None => "[no outputs]".to_string(),
        };
        println!(
            "[{}/end] [{}] [{}] Exiting {} run with output:\n{}",
            run.run_type,
            run.name,
            format_elapsed(run),
            run.run_type,
            outputs_str
        );
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        println!(
            "[{}/error] [{}] [{}] {} run errored:\n{}",
            run.run_type,
            run.name,
            format_elapsed(run),
            run.run_type,
            error
        );
    }

    fn get_runs(&self) -> Vec<Run> {
        Vec::new()
    }

    fn get_run(&self, _run_id: Uuid) -> Option<Run> {
        None
    }
}

impl Clone for StdoutTracer {
    fn clone(&self) -> Self {
        Self
    }
}

// ---------------------------------------------------------------------------
// LogStreamTracer
// ---------------------------------------------------------------------------

/// Tracer that streams run logs as structured JSON lines.
///
/// Each lifecycle event is serialized as a single JSON object containing
/// the event type, run metadata, and optional payload. This tracer is
/// useful for piping trace data to external consumers that expect
/// newline-delimited JSON.
#[derive(Debug)]
pub struct LogStreamTracer {
    buffer: Arc<RwLock<Vec<String>>>,
}

impl LogStreamTracer {
    /// Creates a new `LogStreamTracer`.
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Emits a structured JSON log entry for a run event.
    async fn emit(&self, event_type: &str, run: &Run, extra: Option<&Value>) {
        let mut entry = serde_json::Map::new();
        entry.insert("event".to_string(), Value::String(event_type.to_string()));
        entry.insert(
            "run_id".to_string(),
            Value::String(run.id.to_string()),
        );
        entry.insert("name".to_string(), Value::String(run.name.clone()));
        entry.insert("run_type".to_string(), Value::String(run.run_type.to_string()));
        entry.insert(
            "status".to_string(),
            Value::String(run.status.to_string()),
        );
        entry.insert(
            "start_time".to_string(),
            Value::String(run.start_time.to_rfc3339()),
        );
        if let Some(end) = run.end_time {
            entry.insert("end_time".to_string(), Value::String(end.to_rfc3339()));
        }
        if let Some(ref err) = run.error {
            entry.insert("error".to_string(), Value::String(err.clone()));
        }
        if let Some(data) = extra {
            entry.insert("data".to_string(), data.clone());
        }
        let line = serde_json::to_string(&Value::Object(entry))
            .unwrap_or_else(|_| "{}".to_string());
        self.buffer.write().await.push(line);
    }

    /// Drains and returns all buffered JSON log lines.
    pub async fn drain(&self) -> Vec<String> {
        self.buffer.write().await.drain(..).collect()
    }

    /// Returns the number of buffered log entries.
    pub async fn len(&self) -> usize {
        self.buffer.read().await.len()
    }

    /// Returns `true` if no log entries have been buffered.
    pub async fn is_empty(&self) -> bool {
        self.buffer.read().await.is_empty()
    }
}

impl Default for LogStreamTracer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTracer for LogStreamTracer {
    async fn on_run_create(&self, run: &Run) {
        self.emit("run_create", run, Some(&run.inputs)).await;
    }

    async fn on_run_update(&self, run: &Run) {
        self.emit("run_update", run, None).await;
    }

    async fn on_run_end(&self, run: &Run) {
        let payload = run.outputs.as_ref().cloned().unwrap_or(Value::Null);
        self.emit("run_end", run, Some(&payload)).await;
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        self.emit(
            "run_error",
            run,
            Some(&Value::String(error.to_string())),
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

impl Clone for LogStreamTracer {
    fn clone(&self) -> Self {
        Self {
            buffer: Arc::clone(&self.buffer),
        }
    }
}

// ---------------------------------------------------------------------------
// TracerContext
// ---------------------------------------------------------------------------

/// Shared reference to the currently active tracer.
///
/// Provides a mechanism analogous to thread-local or context-variable
/// storage in the Python implementation. The context holds an
/// `Arc<dyn BaseTracer>` that other parts of the system can query to
/// determine the active tracer.
#[derive(Clone)]
pub struct TracerContext {
    tracer: Arc<dyn BaseTracer>,
}

impl TracerContext {
    /// Creates a new `TracerContext` wrapping the given tracer.
    pub fn new(tracer: Arc<dyn BaseTracer>) -> Self {
        Self { tracer }
    }

    /// Returns a reference to the inner tracer.
    pub fn tracer(&self) -> &Arc<dyn BaseTracer> {
        &self.tracer
    }

    /// Returns a cloned `Arc` to the inner tracer.
    pub fn into_inner(self) -> Arc<dyn BaseTracer> {
        self.tracer
    }
}

impl std::fmt::Debug for TracerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TracerContext").finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// RootListeners
// ---------------------------------------------------------------------------

/// Listener callbacks invoked on root-level run events.
///
/// `RootListeners` wraps optional closures that fire when a root run
/// (one without a `parent_run_id`) is created, ends, or errors.
/// This mirrors the Python `RootListenersTracer` pattern.
pub struct RootListeners {
    /// Called when a root run starts.
    pub on_start: Option<Arc<dyn Fn(&Run) + Send + Sync>>,
    /// Called when a root run ends successfully.
    pub on_end: Option<Arc<dyn Fn(&Run) + Send + Sync>>,
    /// Called when a root run errors.
    pub on_error: Option<Arc<dyn Fn(&Run, &str) + Send + Sync>>,
    root_id: Arc<RwLock<Option<Uuid>>>,
}

impl std::fmt::Debug for RootListeners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootListeners")
            .field("on_start", &self.on_start.is_some())
            .field("on_end", &self.on_end.is_some())
            .field("on_error", &self.on_error.is_some())
            .field("root_id", &self.root_id)
            .finish()
    }
}

impl RootListeners {
    /// Creates a new `RootListeners` with the given callbacks.
    pub fn new(
        on_start: Option<Arc<dyn Fn(&Run) + Send + Sync>>,
        on_end: Option<Arc<dyn Fn(&Run) + Send + Sync>>,
        on_error: Option<Arc<dyn Fn(&Run, &str) + Send + Sync>>,
    ) -> Self {
        Self {
            on_start,
            on_end,
            on_error,
            root_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the root run ID, if a root run has been observed.
    pub async fn root_id(&self) -> Option<Uuid> {
        *self.root_id.read().await
    }
}

impl Clone for RootListeners {
    fn clone(&self) -> Self {
        Self {
            on_start: self.on_start.clone(),
            on_end: self.on_end.clone(),
            on_error: self.on_error.clone(),
            root_id: Arc::clone(&self.root_id),
        }
    }
}

#[async_trait]
impl BaseTracer for RootListeners {
    async fn on_run_create(&self, run: &Run) {
        if run.parent_run_id.is_some() {
            return;
        }
        let mut root_id = self.root_id.write().await;
        if root_id.is_some() {
            return;
        }
        *root_id = Some(run.id);
        drop(root_id);
        if let Some(ref on_start) = self.on_start {
            on_start(run);
        }
    }

    async fn on_run_update(&self, _run: &Run) {}

    async fn on_run_end(&self, run: &Run) {
        let root_id = self.root_id.read().await;
        let is_root = *root_id == Some(run.id);
        drop(root_id);
        if !is_root {
            return;
        }
        if run.error.is_none() {
            if let Some(ref on_end) = self.on_end {
                on_end(run);
            }
        }
    }

    async fn on_run_error(&self, run: &Run, error: &str) {
        let root_id = self.root_id.read().await;
        let is_root = *root_id == Some(run.id);
        drop(root_id);
        if !is_root {
            return;
        }
        if let Some(ref on_error) = self.on_error {
            on_error(run, error);
        }
    }

    fn get_runs(&self) -> Vec<Run> {
        Vec::new()
    }

    fn get_run(&self, _run_id: Uuid) -> Option<Run> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_serialization_roundtrip() {
        let mut run = Run::new(Uuid::new_v4(), "test", RunType::Chain, None, serde_json::json!({"input": "hello"}));
        run.succeed(serde_json::json!({"output": "world"}));
        let json = serde_json::to_string(&run).unwrap();
        let deserialized: Run = serde_json::from_str(&json).unwrap();
        assert_eq!(run.id, deserialized.id);
        assert_eq!(run.name, deserialized.name);
        assert_eq!(run.status, deserialized.status);
        assert_eq!(run.inputs, deserialized.inputs);
        assert_eq!(run.outputs, deserialized.outputs);
    }

    #[test]
    fn test_run_with_child_serialization() {
        let mut parent = Run::new(Uuid::new_v4(), "parent", RunType::Chain, None, Value::Null);
        let child = Run::new(Uuid::new_v4(), "child", RunType::Tool, Some(parent.id), Value::Null);
        parent.add_child_run(child);
        let json = serde_json::to_string(&parent).unwrap();
        let deserialized: Run = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.child_runs.len(), 1);
    }

    #[test]
    fn test_event_serialization() {
        let event = Event { name: "test".into(), time: Utc::now(), data: Some(serde_json::json!({"key": "val"})) };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event.name, deserialized.name);
    }

    #[test]
    fn test_run_tags_and_metadata() {
        let mut run = Run::new(Uuid::new_v4(), "tagged", RunType::Chain, None, Value::Null);
        run.tags.push("important".into());
        run.metadata = serde_json::json!({"env": "test"});
        assert!(run.tags.contains(&"important".to_string()));
        assert_eq!(run.metadata["env"], "test");
    }

    #[test]
    fn test_run_type_serialization() {
        let types = vec![RunType::Llm, RunType::ChatModel, RunType::Chain, RunType::Tool, RunType::Retriever];
        for rt in types {
            let json = serde_json::to_string(&rt).unwrap();
            let deserialized: RunType = serde_json::from_str(&json).unwrap();
            assert_eq!(rt, deserialized);
        }
    }

    #[test]
    fn test_run_status_serialization() {
        let statuses = vec![RunStatus::NotStarted, RunStatus::Running, RunStatus::Succeeded, RunStatus::Failed, RunStatus::Cancelled];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let deserialized: RunStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, deserialized);
        }
    }

    #[test]
    fn run_new_is_running() {
        let run = Run::new(
            Uuid::new_v4(),
            "test",
            RunType::Chain,
            None,
            Value::Null,
        );
        assert_eq!(run.status, RunStatus::Running);
        assert!(run.end_time.is_none());
        assert_eq!(run.events.len(), 1);
        assert_eq!(run.events[0].name, "start");
    }

    #[test]
    fn run_succeed_sets_status() {
        let mut run = Run::new(
            Uuid::new_v4(),
            "test",
            RunType::Llm,
            None,
            Value::Null,
        );
        run.succeed(serde_json::json!({"result": "ok"}));
        assert_eq!(run.status, RunStatus::Succeeded);
        assert!(run.end_time.is_some());
        assert!(run.outputs.is_some());
        assert!(run.error.is_none());
    }

    #[test]
    fn run_fail_sets_error() {
        let mut run = Run::new(
            Uuid::new_v4(),
            "test",
            RunType::Tool,
            None,
            Value::Null,
        );
        run.fail("something broke");
        assert_eq!(run.status, RunStatus::Failed);
        assert!(run.end_time.is_some());
        assert_eq!(run.error.as_deref(), Some("something broke"));
    }

    #[test]
    fn run_cancel_sets_status() {
        let mut run = Run::new(
            Uuid::new_v4(),
            "test",
            RunType::Agent,
            None,
            Value::Null,
        );
        run.cancel();
        assert_eq!(run.status, RunStatus::Cancelled);
        assert!(run.end_time.is_some());
    }

    #[test]
    fn run_elapsed_returns_duration() {
        let mut run = Run::new(
            Uuid::new_v4(),
            "test",
            RunType::Chain,
            None,
            Value::Null,
        );
        assert!(run.elapsed().is_none());
        run.succeed(Value::Null);
        assert!(run.elapsed().is_some());
    }

    #[test]
    fn run_add_child() {
        let mut parent = Run::new(
            Uuid::new_v4(),
            "parent",
            RunType::Chain,
            None,
            Value::Null,
        );
        let child = Run::new(
            Uuid::new_v4(),
            "child",
            RunType::Tool,
            Some(parent.id),
            Value::Null,
        );
        parent.add_child_run(child);
        assert_eq!(parent.child_runs.len(), 1);
    }

    #[test]
    fn run_type_display() {
        assert_eq!(RunType::Llm.to_string(), "llm");
        assert_eq!(RunType::ChatModel.to_string(), "chat_model");
        assert_eq!(RunType::Chain.to_string(), "chain");
        assert_eq!(RunType::Tool.to_string(), "tool");
        assert_eq!(RunType::Retriever.to_string(), "retriever");
        assert_eq!(RunType::Agent.to_string(), "agent");
        assert_eq!(RunType::Memory.to_string(), "memory");
        assert_eq!(RunType::Callback.to_string(), "callback");
        assert_eq!(RunType::Prompt.to_string(), "prompt");
        assert_eq!(RunType::Other.to_string(), "other");
    }

    #[test]
    fn run_status_display() {
        assert_eq!(RunStatus::NotStarted.to_string(), "not_started");
        assert_eq!(RunStatus::Running.to_string(), "running");
        assert_eq!(RunStatus::Succeeded.to_string(), "succeeded");
        assert_eq!(RunStatus::Failed.to_string(), "failed");
        assert_eq!(RunStatus::Cancelled.to_string(), "cancelled");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn langchain_tracer_tracks_runs() {
        let tracer = LangChainTracer::new(Some("test-project".to_string()), vec![]);
        let run = Run::new(
            Uuid::new_v4(),
            "test-run",
            RunType::Chain,
            None,
            Value::Null,
        );
        let run_id = run.id;
        tracer.on_run_create(&run).await;
        assert_eq!(tracer.get_runs().len(), 1);
        assert!(tracer.get_run(run_id).is_some());
        let latest = tracer.latest_run_id().await;
        assert_eq!(latest, Some(run_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn langchain_tracer_flush_drains() {
        let tracer = LangChainTracer::new(None, vec![]);
        let run = Run::new(
            Uuid::new_v4(),
            "test-run",
            RunType::Llm,
            None,
            Value::Null,
        );
        tracer.on_run_create(&run).await;
        let flushed = tracer.flush().await;
        assert_eq!(flushed.len(), 1);
        assert!(tracer.get_runs().is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn run_collector_collects() {
        let collector = RunCollector::new(None);
        let run = Run::new(
            Uuid::new_v4(),
            "collected",
            RunType::Tool,
            None,
            Value::Null,
        );
        collector.on_run_create(&run).await;
        assert_eq!(collector.get_runs().len(), 1);
        assert!(collector.get_run(run.id).is_some());
    }

    #[tokio::test]
    async fn run_collector_drain() {
        let collector = RunCollector::new(None);
        let run = Run::new(
            Uuid::new_v4(),
            "collected",
            RunType::Tool,
            None,
            Value::Null,
        );
        collector.on_run_create(&run).await;
        let drained = collector.drain().await;
        assert_eq!(drained.len(), 1);
        assert!(collector.is_empty().await);
    }

    #[tokio::test]
    async fn evaluation_tracer_skips_unfinished() {
        let tracer = EvaluationTracer::new(true);
        let mut run = Run::new(
            Uuid::new_v4(),
            "eval",
            RunType::Chain,
            None,
            Value::Null,
        );
        tracer.on_run_end(&run).await;
        assert!(tracer.evaluation_runs().await.is_empty());
        run.succeed(Value::Null);
        tracer.on_run_end(&run).await;
        assert_eq!(tracer.evaluation_runs().await.len(), 1);
    }

    #[tokio::test]
    async fn evaluation_tracer_includes_unfinished_when_configured() {
        let tracer = EvaluationTracer::new(false);
        let run = Run::new(
            Uuid::new_v4(),
            "eval",
            RunType::Chain,
            None,
            Value::Null,
        );
        tracer.on_run_end(&run).await;
        assert_eq!(tracer.evaluation_runs().await.len(), 1);
    }

    #[tokio::test]
    async fn log_stream_tracer_buffers_lines() {
        let tracer = LogStreamTracer::new();
        let run = Run::new(
            Uuid::new_v4(),
            "stream-test",
            RunType::Llm,
            None,
            Value::Null,
        );
        tracer.on_run_create(&run).await;
        assert_eq!(tracer.len().await, 1);
        let lines = tracer.drain().await;
        assert_eq!(lines.len(), 1);
        let parsed: Value = serde_json::from_str(&lines[0]).expect("valid json");
        assert_eq!(parsed["event"], "run_create");
    }

    #[tokio::test]
    async fn root_listeners_fire_on_root_run() {
        let started = Arc::new(std::sync::Mutex::new(false));
        let ended = Arc::new(std::sync::Mutex::new(false));
        let started_clone = Arc::clone(&started);
        let ended_clone = Arc::clone(&ended);
        let listeners = RootListeners::new(
            Some(Arc::new(move |_run: &Run| {
                *started_clone.lock().unwrap() = true;
            })),
            Some(Arc::new(move |_run: &Run| {
                *ended_clone.lock().unwrap() = true;
            })),
            None,
        );
        let mut run = Run::new(
            Uuid::new_v4(),
            "root",
            RunType::Chain,
            None,
            Value::Null,
        );
        listeners.on_run_create(&run).await;
        assert!(*started.lock().unwrap());
        run.succeed(Value::Null);
        listeners.on_run_end(&run).await;
        assert!(*ended.lock().unwrap());
    }

    #[tokio::test]
    async fn root_listeners_ignores_child_runs() {
        let started = Arc::new(std::sync::Mutex::new(0u32));
        let started_clone = Arc::clone(&started);
        let listeners = RootListeners::new(
            Some(Arc::new(move |_run: &Run| {
                *started_clone.lock().unwrap() += 1;
            })),
            None,
            None,
        );
        let root_run = Run::new(
            Uuid::new_v4(),
            "root",
            RunType::Chain,
            None,
            Value::Null,
        );
        let child_run = Run::new(
            Uuid::new_v4(),
            "child",
            RunType::Tool,
            Some(root_run.id),
            Value::Null,
        );
        listeners.on_run_create(&root_run).await;
        listeners.on_run_create(&child_run).await;
        assert_eq!(*started.lock().unwrap(), 1);
    }
}
