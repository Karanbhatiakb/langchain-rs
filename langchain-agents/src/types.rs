//! Types for agent actions, finish states, steps, and intermediate values.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// An action to invoke a tool with specific input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    /// The name of the tool to invoke.
    pub tool: String,
    /// The input to pass to the tool.
    pub tool_input: String,
    /// A log message describing this action.
    pub log: String,
}

impl AgentAction {
    /// Creates a new `AgentAction`.
    pub fn new(tool: impl Into<String>, tool_input: impl Into<String>, log: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            tool_input: tool_input.into(),
            log: log.into(),
        }
    }
}

/// The final output of an agent's execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFinish {
    /// The final return values (key-value map).
    pub return_values: HashMap<String, Value>,
    /// A log message describing the finish.
    pub log: String,
}

impl AgentFinish {
    /// Creates a new `AgentFinish`.
    pub fn new(return_values: HashMap<String, Value>, log: impl Into<String>) -> Self {
        Self {
            return_values,
            log: log.into(),
        }
    }
}

/// A single step in an agent's execution trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    /// The action that was taken.
    pub action: AgentAction,
    /// The observation returned by the tool.
    pub observation: String,
}

impl AgentStep {
    /// Creates a new `AgentStep`.
    pub fn new(action: AgentAction, observation: impl Into<String>) -> Self {
        Self {
            action,
            observation: observation.into(),
        }
    }
}

/// Alias for [`AgentStep`], used for intermediate steps during planning.
pub type IntermediateStep = AgentStep;

/// The result of an agent's planning step — either an action or a finish.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentNextStep {
    /// The agent has decided to invoke a tool.
    Action(AgentAction),
    /// The agent has finished and produced output.
    Finish(AgentFinish),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_action_new() {
        let action = AgentAction::new("search", "query", "searching...");
        assert_eq!(action.tool, "search");
        assert_eq!(action.tool_input, "query");
        assert_eq!(action.log, "searching...");
    }

    #[test]
    fn test_agent_action_serialization() {
        let action = AgentAction::new("tool", "input", "log");
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: AgentAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool, "tool");
    }

    #[test]
    fn test_agent_finish_new() {
        let mut map = HashMap::new();
        map.insert("output".into(), Value::String("done".into()));
        let finish = AgentFinish::new(map, "finished");
        assert_eq!(finish.log, "finished");
        assert_eq!(finish.return_values.get("output").and_then(|v| v.as_str()), Some("done"));
    }

    #[test]
    fn test_agent_finish_serialization() {
        let mut map = HashMap::new();
        map.insert("output".into(), Value::String("result".into()));
        let finish = AgentFinish::new(map, "done");
        let json = serde_json::to_string(&finish).unwrap();
        let deserialized: AgentFinish = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.log, "done");
    }

    #[test]
    fn test_agent_step_new() {
        let action = AgentAction::new("tool", "input", "log");
        let step = AgentStep::new(action.clone(), "observation");
        assert_eq!(step.action.tool, "tool");
        assert_eq!(step.observation, "observation");
    }

    #[test]
    fn test_agent_step_serialization() {
        let action = AgentAction::new("t", "i", "l");
        let step = AgentStep::new(action, "obs");
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: AgentStep = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.observation, "obs");
    }

    #[test]
    fn test_agent_next_step_action() {
        let action = AgentAction::new("tool", "input", "log");
        let step = AgentNextStep::Action(action);
        match step {
            AgentNextStep::Action(a) => assert_eq!(a.tool, "tool"),
            _ => panic!("expected Action"),
        }
    }

    #[test]
    fn test_agent_next_step_finish() {
        let finish = AgentFinish::new(HashMap::new(), "done");
        let step = AgentNextStep::Finish(finish);
        match step {
            AgentNextStep::Finish(f) => assert_eq!(f.log, "done"),
            _ => panic!("expected Finish"),
        }
    }

    #[test]
    fn test_intermediate_step_alias() {
        let action = AgentAction::new("t", "i", "l");
        let step: IntermediateStep = AgentStep::new(action, "obs");
        assert_eq!(step.observation, "obs");
    }

    #[test]
    fn test_types_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AgentAction>();
        assert_sync::<AgentAction>();
        assert_send::<AgentFinish>();
        assert_sync::<AgentFinish>();
        assert_send::<AgentStep>();
        assert_sync::<AgentStep>();
        assert_send::<AgentNextStep>();
        assert_sync::<AgentNextStep>();
    }
}
