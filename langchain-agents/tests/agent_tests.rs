use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_llms::fake::FakeListLLM;
use langchain_tools::traits::BaseTool;
use serde_json::Value;

use langchain_agents::types::{AgentAction, AgentFinish, AgentNextStep, AgentStep};
use langchain_agents::traits::Agent;
use langchain_agents::react::ReActAgent;

struct FakeTool {
    name_str: String,
    desc_str: String,
}

impl FakeTool {
    fn new(name: &str, desc: &str) -> Self {
        Self {
            name_str: name.to_string(),
            desc_str: desc.to_string(),
        }
    }
}

#[async_trait]
impl BaseTool for FakeTool {
    fn name(&self) -> &str { &self.name_str }
    fn description(&self) -> &str { &self.desc_str }
    async fn invoke(&self, _input: &str) -> Result<String> {
        Ok(format!("result from {}", self.name_str))
    }
}

#[test]
fn test_agent_action_new() {
    let action = AgentAction::new("search", "query", "thought log");
    assert_eq!(action.tool, "search");
    assert_eq!(action.tool_input, "query");
    assert_eq!(action.log, "thought log");
}

#[test]
fn test_agent_finish_new() {
    let mut return_values = HashMap::new();
    return_values.insert("output".to_string(), Value::String("done".to_string()));
    let finish = AgentFinish::new(return_values, "final log");
    assert_eq!(finish.return_values.get("output").unwrap(), "done");
    assert_eq!(finish.log, "final log");
}

#[test]
fn test_agent_step_new() {
    let action = AgentAction::new("tool", "input", "log");
    let step = AgentStep::new(action, "observation");
    assert_eq!(step.action.tool, "tool");
    assert_eq!(step.observation, "observation");
}

#[test]
fn test_agent_next_step_variants() {
    let action = AgentAction::new("search", "q", "log");
    let next = AgentNextStep::Action(action);
    assert!(matches!(next, AgentNextStep::Action(_)));

    let finish = AgentFinish::new(HashMap::new(), "log");
    let next = AgentNextStep::Finish(finish);
    assert!(matches!(next, AgentNextStep::Finish(_)));
}

#[test]
fn test_agent_action_serialization() {
    let action = AgentAction::new("search", "query", "log");
    let json = serde_json::to_string(&action).unwrap();
    let deserialized: AgentAction = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool, "search");
}

#[test]
fn test_agent_finish_serialization() {
    let mut rv = HashMap::new();
    rv.insert("output".to_string(), Value::String("result".to_string()));
    let finish = AgentFinish::new(rv, "log");
    let json = serde_json::to_string(&finish).unwrap();
    let deserialized: AgentFinish = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.log, "log");
}

#[tokio::test]
async fn test_react_agent_parse_final_answer() {
    let llm = Arc::new(FakeListLLM::new(vec![
        "Thought: I know the answer\nFinal Answer: 42".to_string(),
    ]));
    let tools: Vec<Arc<dyn BaseTool>> = vec![
        Arc::new(FakeTool::new("search", "Search tool")),
    ];
    let agent = ReActAgent::new(llm, tools, None);
    assert_eq!(agent.input_keys(), vec!["input".to_string()]);
    assert_eq!(agent.output_keys(), vec!["output".to_string()]);
}

#[tokio::test]
async fn test_react_agent_plan_final_answer() {
    let llm = Arc::new(FakeListLLM::new(vec![
        "Thought: I know\nFinal Answer: hello".to_string(),
    ]));
    let tools: Vec<Arc<dyn BaseTool>> = vec![
        Arc::new(FakeTool::new("calc", "Calculator")),
    ];
    let agent = ReActAgent::new(llm, tools, None);
    let mut inputs = HashMap::new();
    inputs.insert("input".to_string(), Value::String("What is 2+2?".to_string()));
    let result = agent.plan(&[], &inputs).await.unwrap();
    match result {
        AgentNextStep::Finish(finish) => {
            let output = finish.return_values.get("output").unwrap();
            assert_eq!(output.as_str().unwrap(), "hello");
        }
        AgentNextStep::Action(_) => panic!("Expected Finish, got Action"),
    }
}

#[tokio::test]
async fn test_react_agent_plan_action() {
    let llm = Arc::new(FakeListLLM::new(vec![
        "Thought: need to search\nAction: search\nAction Input: weather".to_string(),
    ]));
    let tools: Vec<Arc<dyn BaseTool>> = vec![
        Arc::new(FakeTool::new("search", "Search tool")),
    ];
    let agent = ReActAgent::new(llm, tools, None);
    let mut inputs = HashMap::new();
    inputs.insert("input".to_string(), Value::String("What is the weather?".to_string()));
    let result = agent.plan(&[], &inputs).await.unwrap();
    match result {
        AgentNextStep::Action(action) => {
            assert_eq!(action.tool, "search");
            assert_eq!(action.tool_input, "weather");
        }
        AgentNextStep::Finish(_) => panic!("Expected Action, got Finish"),
    }
}

#[tokio::test]
async fn test_react_agent_return_stopped_response_force() {
    let llm = Arc::new(FakeListLLM::new(vec!["answer".to_string()]));
    let tools: Vec<Arc<dyn BaseTool>> = vec![
        Arc::new(FakeTool::new("search", "Search tool")),
    ];
    let agent = ReActAgent::new(llm, tools, None);
    let inputs = HashMap::new();
    let result = agent.return_stopped_response("force", &[], &inputs).await.unwrap();
    assert!(result.return_values.contains_key("output"));
}

#[tokio::test]
async fn test_react_agent_with_max_tokens() {
    let llm = Arc::new(FakeListLLM::new(vec!["test".to_string()]));
    let tools: Vec<Arc<dyn BaseTool>> = vec![];
    let _agent = ReActAgent::new(llm, tools, None).with_max_tokens(100);
}

#[tokio::test]
async fn test_react_agent_custom_prompt() {
    use langchain_core::prompt::PromptTemplate;
    let llm = Arc::new(FakeListLLM::new(vec![
        "Thought: done\nFinal Answer: custom".to_string(),
    ]));
    let tools: Vec<Arc<dyn BaseTool>> = vec![];
    let prompt = PromptTemplate::from_template("Custom: {input}\n{agent_scratchpad}");
    let agent = ReActAgent::new(llm, tools, Some(prompt));
    let mut inputs = HashMap::new();
    inputs.insert("input".to_string(), Value::String("test".to_string()));
    let result = agent.plan(&[], &inputs).await.unwrap();
    match result {
        AgentNextStep::Finish(f) => {
            assert_eq!(f.return_values.get("output").unwrap().as_str().unwrap(), "custom");
        }
        AgentNextStep::Action(_) => panic!("Expected Finish"),
    }
}
