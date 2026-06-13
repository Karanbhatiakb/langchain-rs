//! Agent trajectory evaluators — evaluate agent tool-use trajectories.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::traits::{EvaluationResult, Evaluator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub input: Value,
    pub output: Option<Value>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrajectory {
    pub input: String,
    pub tool_calls: Vec<ToolCall>,
    pub final_output: String,
    pub total_steps: usize,
}

impl AgentTrajectory {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            tool_calls: Vec::new(),
            final_output: String::new(),
            total_steps: 0,
        }
    }

    pub fn with_tool_call(mut self, tool_call: ToolCall) -> Self {
        self.total_steps += 1;
        self.tool_calls.push(tool_call);
        self
    }

    pub fn with_final_output(mut self, output: impl Into<String>) -> Self {
        self.final_output = output.into();
        self
    }
}

pub struct TrajectoryEvalChain {
    evaluate_order: bool,
    evaluate_efficiency: bool,
    evaluate_completion: bool,
}

impl TrajectoryEvalChain {
    pub fn new() -> Self {
        Self {
            evaluate_order: true,
            evaluate_efficiency: true,
            evaluate_completion: true,
        }
    }

    pub fn with_order_evaluation(mut self, enabled: bool) -> Self {
        self.evaluate_order = enabled;
        self
    }

    pub fn with_efficiency_evaluation(mut self, enabled: bool) -> Self {
        self.evaluate_efficiency = enabled;
        self
    }

    pub fn with_completion_evaluation(mut self, enabled: bool) -> Self {
        self.evaluate_completion = enabled;
        self
    }
}

impl Default for TrajectoryEvalChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Evaluator<String, AgentTrajectory> for TrajectoryEvalChain {
    fn name(&self) -> &str {
        "trajectory_eval"
    }

    async fn evaluate(
        &self,
        input: String,
        trajectory: AgentTrajectory,
        reference: Option<AgentTrajectory>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let mut scores = HashMap::new();
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        if self.evaluate_completion {
            let completion_score = if trajectory.final_output.is_empty() {
                0.0
            } else {
                let has_relevant_output = trajectory.final_output.to_lowercase().contains(&input.to_lowercase().split_whitespace().next().unwrap_or("").to_lowercase());
                if has_relevant_output { 0.9 } else { 0.5 }
            };
            scores.insert("completion".to_string(), completion_score);
            weighted_sum += completion_score * 0.4;
            total_weight += 0.4;
        }

        if self.evaluate_efficiency {
            let efficiency_score = if trajectory.tool_calls.is_empty() {
                if trajectory.final_output.is_empty() { 0.0 } else { 0.8 }
            } else {
                let success_rate = trajectory.tool_calls.iter().filter(|tc| tc.success).count() as f64
                    / trajectory.tool_calls.len() as f64;
                let optimal_steps = reference.as_ref().map(|r| r.tool_calls.len()).unwrap_or(1).max(1);
                let step_efficiency = if trajectory.tool_calls.len() <= optimal_steps {
                    1.0
                } else {
                    optimal_steps as f64 / trajectory.tool_calls.len() as f64
                };
                0.6 * success_rate + 0.4 * step_efficiency
            };
            scores.insert("efficiency".to_string(), efficiency_score);
            weighted_sum += efficiency_score * 0.35;
            total_weight += 0.35;
        }

        if self.evaluate_order {
            let order_score = compute_tool_order_score(&trajectory, reference.as_ref());
            scores.insert("order".to_string(), order_score);
            weighted_sum += order_score * 0.25;
            total_weight += 0.25;
        }

        let overall = if total_weight > 0.0 { weighted_sum / total_weight } else { 0.0 };
        let label = if overall >= 0.7 { "good_trajectory" } else if overall >= 0.4 { "acceptable_trajectory" } else { "poor_trajectory" };

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("tool_count".to_string(), Value::from(trajectory.tool_calls.len() as u64));
        metadata.insert("total_steps".to_string(), Value::from(trajectory.total_steps as u64));
        metadata.insert("successful_tools".to_string(), Value::from(trajectory.tool_calls.iter().filter(|tc| tc.success).count() as u64));
        metadata.insert("trajectory_scores".to_string(), {
            let mut m = serde_json::Map::new();
            for (k, v) in &scores {
                m.insert(k.clone(), Value::Number(serde_json::Number::from_f64(*v).unwrap_or_else(|| serde_json::Number::from(0))));
            }
            Value::Object(m)
        });

        Ok(EvaluationResult::new(overall)
            .with_label(label)
            .with_metadata(metadata))
    }
}

fn compute_tool_order_score(trajectory: &AgentTrajectory, reference: Option<&AgentTrajectory>) -> f64 {
    if trajectory.tool_calls.len() <= 1 {
        return 1.0;
    }

    let search_tools: Vec<&str> = ["search", "lookup", "retrieve", "find", "query"].iter().copied().collect();
    let action_tools: Vec<&str> = ["calculator", "execute", "run", "compute", "process"].iter().copied().collect();

    let mut search_before_action = true;
    let mut last_search_idx = None;
    let mut first_action_idx = None;

    for (i, tc) in trajectory.tool_calls.iter().enumerate() {
        let name_lower = tc.tool_name.to_lowercase();
        if search_tools.iter().any(|s| name_lower.contains(s)) {
            last_search_idx = Some(i);
        }
        if action_tools.iter().any(|a| name_lower.contains(a)) && first_action_idx.is_none() {
            first_action_idx = Some(i);
        }
    }

    if let (Some(search_idx), Some(action_idx)) = (last_search_idx, first_action_idx) {
        search_before_action = search_idx < action_idx;
    }

    let base_score = if search_before_action { 1.0 } else { 0.3 };

    match reference {
        Some(ref_traj) => {
            if trajectory.tool_calls.len() != ref_traj.tool_calls.len() {
                let len_ratio = trajectory.tool_calls.len().min(ref_traj.tool_calls.len()) as f64
                    / trajectory.tool_calls.len().max(ref_traj.tool_calls.len()) as f64;
                base_score * len_ratio
            } else {
                let matching_positions = trajectory.tool_calls.iter().zip(ref_traj.tool_calls.iter())
                    .filter(|(a, b)| a.tool_name == b.tool_name)
                    .count();
                base_score * (matching_positions as f64 / trajectory.tool_calls.len() as f64)
            }
        }
        None => base_score,
    }
}

pub struct ToolSelectionEvaluator {
    available_tools: Vec<String>,
}

impl ToolSelectionEvaluator {
    pub fn new(available_tools: Vec<String>) -> Self {
        Self { available_tools }
    }

    pub fn with_default_tools() -> Self {
        Self::new(vec![
            "search".to_string(),
            "calculator".to_string(),
            "lookup".to_string(),
            "execute".to_string(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSelectionInput {
    pub query: String,
    pub selected_tools: Vec<String>,
    pub expected_tools: Option<Vec<String>>,
}

#[async_trait]
impl Evaluator<String, ToolSelectionInput> for ToolSelectionEvaluator {
    fn name(&self) -> &str {
        "tool_selection"
    }

    async fn evaluate(
        &self,
        input: String,
        prediction: ToolSelectionInput,
        _reference: Option<ToolSelectionInput>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let selected: std::collections::HashSet<&str> = prediction.selected_tools.iter().map(|s| s.as_str()).collect();
        let available: std::collections::HashSet<&str> = self.available_tools.iter().map(|s| s.as_str()).collect();

        let all_valid = selected.iter().all(|s| available.contains(s));
        let valid_ratio = if selected.is_empty() {
            0.0
        } else {
            selected.intersection(&available).count() as f64 / selected.len() as f64
        };

        let (precision, recall) = match &prediction.expected_tools {
            Some(expected) => {
                let expected_set: std::collections::HashSet<&str> = expected.iter().map(|s| s.as_str()).collect();
                let intersection = selected.intersection(&expected_set).count() as f64;
                let prec = if selected.is_empty() { 0.0 } else { intersection / selected.len() as f64 };
                let rec = if expected_set.is_empty() { 1.0 } else { intersection / expected_set.len() as f64 };
                (prec, rec)
            }
            None => (valid_ratio, 1.0),
        };

        let f1 = if precision + recall == 0.0 { 0.0 } else { 2.0 * precision * recall / (precision + recall) };

        let label = if f1 >= 0.8 { "good_selection" } else if f1 >= 0.5 { "partial_selection" } else { "poor_selection" };

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("selected_tools".to_string(), Value::Array(prediction.selected_tools.iter().map(|s| Value::String(s.clone())).collect()));
        metadata.insert("precision".to_string(), Value::Number(serde_json::Number::from_f64(precision).unwrap_or_else(|| serde_json::Number::from(0))));
        metadata.insert("recall".to_string(), Value::Number(serde_json::Number::from_f64(recall).unwrap_or_else(|| serde_json::Number::from(0))));
        metadata.insert("all_valid".to_string(), Value::Bool(all_valid));

        Ok(EvaluationResult::new(f1)
            .with_label(label)
            .with_metadata(metadata))
    }
}
