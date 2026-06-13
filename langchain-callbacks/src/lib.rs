//! Callback handlers for observability, tracing, and logging.
//!
//! Provides the [`CallbackHandler`] trait, a [`CallbackManager`] for
//! dispatching, and implementations for stdout, LangSmith, W&B, MLflow,
//! ClearML, Langfuse, Portkey, Comet, PromptLayer, and Helicone — each gated
//! behind a feature flag.

pub mod traits;
pub mod manager;
pub mod base;
pub mod file;
pub mod streaming_stdout;
pub mod usage;
pub mod run_collector;
pub mod evaluation;
pub mod event_stream;
#[cfg(feature = "stdout")]
pub mod stdout;
#[cfg(feature = "langsmith")]
pub mod langsmith;
#[cfg(feature = "wandb")]
pub mod wandb;
#[cfg(feature = "mlflow")]
pub mod mlflow;
#[cfg(feature = "clearml")]
pub mod clearml;
#[cfg(feature = "langfuse")]
pub mod langfuse;
#[cfg(feature = "portkey")]
pub mod portkey;
#[cfg(feature = "comet")]
pub mod comet;
#[cfg(feature = "promptlayer")]
pub mod promptlayer;
#[cfg(feature = "helicone")]
pub mod helicone;

pub use traits::CallbackHandler;
pub use manager::CallbackManager;
pub use base::{AsyncCallbackHandler, BaseCallbackHandler};
pub use file::FileCallbackHandler;
pub use streaming_stdout::StreamingStdOutCallbackHandler;
pub use usage::UsageCallbackHandler;
pub use run_collector::{RunCollectorCallbackHandler, RunRecord};
pub use evaluation::EvaluationCallbackHandler;
pub use event_stream::EventStreamCallbackHandler;
