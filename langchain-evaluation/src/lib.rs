//! Evaluation framework for measuring LLM output quality.
//!
//! Provides traits, metrics, dataset management, evaluators, and runners for
//! evaluating model predictions against reference outputs.

pub mod traits;
pub mod datasets;
pub mod metrics;
pub mod evaluators;
pub mod runners;
pub mod comparison_evaluators;
pub mod string_evaluators;
pub mod agent_evaluators;
pub mod runner;
pub mod langsmith_integration;

#[cfg(feature = "langsmith")]
pub mod langsmith;

pub use traits::{EvaluationResult, Evaluator};
pub use datasets::{Dataset, DatasetExample};
pub use metrics::{
    ExactMatchMetric, ContainsMetric, RegexMetric, F1ScoreMetric, BLEUScoreMetric,
    ROUGEScoreMetric, PrecisionMetric, RecallMetric, AccuracyMetric, StringDistanceMetric,
    EmbeddingDistanceMetric, LLMAsJudgeMetric,
};
pub use evaluators::{
    QAEvaluator, AgentTrajectoryEvaluator, PairwiseStringEvaluator, ScoreStringEvalChain,
};
pub use runners::{run_evaluation, EvalReport, EvalResult, EvalRunner};
pub use comparison_evaluators::{PairwiseStringEvalChain, PairwiseComparison, PairwiseChatEvalChain, EmbeddedSiblingsEvaluator};
pub use string_evaluators::{QAEvalChain, CriteriaEvalChainV2, Criterion, ScoreStringEvalChainV2, StringDistanceType, StringDistanceEvalChain};
pub use agent_evaluators::{TrajectoryEvalChain, ToolSelectionEvaluator};
pub use runner::{EvaluationRunner, RunnerEvalReport, RunnerEvalResult};
pub use langsmith_integration::{LangSmithConfig, EvaluationSession, LangSmithClient};
