//! Agent action and result schemas.
//!
//! Defines the types used to represent agent decisions, observations, and
//! final return values. Agents use language models to choose a sequence of
//! actions. Each step produces an [`AgentAction`] (tool to invoke and input)
//! and an observation. When the agent reaches a stopping condition it returns
//! an [`AgentFinish`].

use crate::messages::BaseMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a request by an agent to execute a tool.
///
/// The `tool` field names the tool and `tool_input` provides its input. The
/// `log` field carries extra information (e.g., the LLM's raw output) useful
/// for debugging and for reconstructing conversation history in future agent
/// iterations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    /// The name of the tool to execute.
    pub tool: String,
    /// The input to pass to the tool.
    pub tool_input: String,
    /// Additional information to log about the action.
    pub log: String,
}

impl AgentAction {
    /// Creates a new `AgentAction`.
    pub fn new(
        tool: impl Into<String>,
        tool_input: impl Into<String>,
        log: impl Into<String>,
    ) -> Self {
        Self {
            tool: tool.into(),
            tool_input: tool_input.into(),
            log: log.into(),
        }
    }
}

/// Final return value of an agent when it has reached a stopping condition.
///
/// The `return_values` map holds the agent's final output (conventionally
/// under the key `"output"`). The `log` field preserves the full LLM
/// prediction for debugging and observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFinish {
    /// Dictionary of return values from the agent.
    pub return_values: HashMap<String, String>,
    /// Additional information to log about the return value.
    pub log: String,
}

impl AgentFinish {
    /// Creates a new `AgentFinish`.
    pub fn new(
        return_values: HashMap<String, String>,
        log: impl Into<String>,
    ) -> Self {
        Self {
            return_values,
            log: log.into(),
        }
    }
}

/// The output of a single agent step — either an action to execute or a
/// final result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentOutput {
    /// The agent decided to invoke a tool.
    Action(AgentAction),
    /// The agent reached a stopping condition and returned a final value.
    Finish(AgentFinish),
}

/// The result of executing an [`AgentAction`].
///
/// Pairs the action that was taken with the observation returned by the tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    /// The action that was executed.
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

/// An [`AgentAction`] augmented with the full message log from the LLM.
///
/// Useful when the underlying LLM is a chat model that returns structured
/// messages rather than a plain string. The `message_logs` field preserves
/// those messages so the conversation history can be reconstructed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionMessageLog {
    /// The action to execute.
    pub action: AgentAction,
    /// The chat messages produced by the LLM before the action was parsed.
    pub message_logs: Vec<BaseMessage>,
}

impl AgentActionMessageLog {
    /// Creates a new `AgentActionMessageLog`.
    pub fn new(action: AgentAction, message_logs: Vec<BaseMessage>) -> Self {
        Self {
            action,
            message_logs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_action_new() {
        let action = AgentAction::new("search", "rust lang", "I need to search");
        assert_eq!(action.tool, "search");
        assert_eq!(action.tool_input, "rust lang");
        assert_eq!(action.log, "I need to search");
    }

    #[test]
    fn test_agent_finish_new() {
        let rv = HashMap::from([("output".to_string(), "42".to_string())]);
        let finish = AgentFinish::new(rv, "Final Answer: 42");
        assert_eq!(finish.return_values.get("output").map(|s| s.as_str()), Some("42"));
        assert_eq!(finish.log, "Final Answer: 42");
    }

    #[test]
    fn test_agent_step_new() {
        let action = AgentAction::new("calc", "1+1", "thinking");
        let step = AgentStep::new(action, "2");
        assert_eq!(step.observation, "2");
    }

    #[test]
    fn test_agent_action_message_log_new() {
        let action = AgentAction::new("tool", "input", "log");
        let logs = vec![BaseMessage::new("thinking", crate::messages::MessageType::AI)];
        let aml = AgentActionMessageLog::new(action, logs);
        assert_eq!(aml.message_logs.len(), 1);
    }

    #[test]
    fn test_agent_action_serialize_deserialize() {
        let action = AgentAction::new("search", "query", "log");
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: AgentAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool, "search");
        assert_eq!(deserialized.tool_input, "query");
        assert_eq!(deserialized.log, "log");
    }

    #[test]
    fn test_agent_finish_serialize_deserialize() {
        let rv = HashMap::from([("output".into(), "result".into())]);
        let finish = AgentFinish::new(rv, "done");
        let json = serde_json::to_string(&finish).unwrap();
        let deserialized: AgentFinish = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.return_values.get("output").map(|s| s.as_str()),
            Some("result")
        );
    }

    #[test]
    fn test_agent_output_action() {
        let action = AgentAction::new("calc", "2+2", "calculating");
        let output = AgentOutput::Action(action);
        match output {
            AgentOutput::Action(a) => {
                assert_eq!(a.tool, "calc");
                assert_eq!(a.tool_input, "2+2");
            }
            _ => panic!("expected Action"),
        }
    }

    #[test]
    fn test_agent_output_finish() {
        let rv = HashMap::from([("output".into(), "42".into())]);
        let finish = AgentFinish::new(rv, "answer");
        let output = AgentOutput::Finish(finish);
        match output {
            AgentOutput::Finish(f) => {
                assert_eq!(f.log, "answer");
                assert_eq!(f.return_values.get("output").unwrap(), "42");
            }
            _ => panic!("expected Finish"),
        }
    }

    #[test]
    fn test_agent_step_with_empty_observation() {
        let action = AgentAction::new("tool", "input", "log");
        let step = AgentStep::new(action, "");
        assert_eq!(step.observation, "");
    }

    #[test]
    fn test_agent_action_with_empty_fields() {
        let action = AgentAction::new("", "", "");
        assert_eq!(action.tool, "");
        assert_eq!(action.tool_input, "");
        assert_eq!(action.log, "");
    }

    #[test]
    fn test_agent_finish_empty_log() {
        let rv = HashMap::new();
        let finish = AgentFinish::new(rv, "");
        assert!(finish.return_values.is_empty());
        assert_eq!(finish.log, "");
    }

    #[test]
    fn test_agent_action_clone() {
        let a1 = AgentAction::new("tool", "input", "log");
        let a2 = a1.clone();
        assert_eq!(a2.tool, "tool");
    }

    #[test]
    fn test_agent_finish_clone() {
        let rv = HashMap::from([("output".into(), "val".into())]);
        let f1 = AgentFinish::new(rv, "log");
        let f2 = f1.clone();
        assert_eq!(f2.log, "log");
    }

    #[test]
    fn test_agent_step_with_action_clone() {
        let action = AgentAction::new("t", "i", "l");
        let step = AgentStep::new(action.clone(), "obs");
        assert_eq!(step.action.tool, "t");
        assert_eq!(step.action.tool_input, "i");
    }

    #[test]
    fn test_agent_action_serde_roundtrip() {
        let action = AgentAction::new("search", "query", "thinking");
        let json = serde_json::to_string(&action).unwrap();
        let back: AgentAction = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tool, "search");
    }

    #[test]
    fn test_agent_finish_serde_roundtrip() {
        let rv = HashMap::from([("output".into(), "answer".into())]);
        let finish = AgentFinish::new(rv, "log");
        let json = serde_json::to_string(&finish).unwrap();
        let back: AgentFinish = serde_json::from_str(&json).unwrap();
        assert_eq!(back.log, "log");
    }

    #[test]
    fn test_agent_output_debug() {
        let action = AgentAction::new("t", "i", "l");
        let output = AgentOutput::Action(action);
        let debug = format!("{:?}", output);
        assert!(debug.contains("Action"));
    }

    #[test]
    fn test_agent_action_message_log_clone() {
        let action = AgentAction::new("t", "i", "l");
        let logs = vec![BaseMessage::new("m", crate::messages::MessageType::AI)];
        let aml = AgentActionMessageLog::new(action, logs);
        let cloned = aml.clone();
        assert_eq!(cloned.action.tool, "t");
    }

    #[test]
    fn test_agents_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AgentAction>();
        assert_sync::<AgentAction>();
        assert_send::<AgentFinish>();
        assert_sync::<AgentFinish>();
        assert_send::<AgentOutput>();
        assert_sync::<AgentOutput>();
        assert_send::<AgentStep>();
        assert_sync::<AgentStep>();
    }
}
