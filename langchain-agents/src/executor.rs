//! Agent executor implementation.

use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::*;
use langchain_memory::traits::BaseMemory;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

use crate::traits::Agent;
use crate::types::{AgentFinish, AgentNextStep, AgentStep, IntermediateStep};
use langchain_tools::traits::BaseTool;

pub struct AgentExecutor {
    agent: Arc<dyn Agent>,
    tools: Vec<Arc<dyn BaseTool>>,
    max_iterations: usize,
    max_execution_time: Option<Duration>,
    early_stopping_method: String,
    handle_parsing_errors: bool,
    return_intermediate_steps: bool,
    verbose: bool,
    callbacks: Option<CallbackManager>,
    memory: Option<Arc<dyn BaseMemory>>,
    tags: Vec<String>,
    metadata: HashMap<String, Value>,
}

impl AgentExecutor {
    pub fn new(agent: Arc<dyn Agent>, tools: Vec<Arc<dyn BaseTool>>) -> Self {
        Self {
            agent,
            tools,
            max_iterations: 15,
            max_execution_time: None,
            early_stopping_method: "force".to_string(),
            handle_parsing_errors: true,
            return_intermediate_steps: false,
            verbose: false,
            callbacks: None,
            memory: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    pub fn with_max_execution_time(mut self, d: Duration) -> Self {
        self.max_execution_time = Some(d);
        self
    }

    pub fn with_early_stopping_method(mut self, method: impl Into<String>) -> Self {
        self.early_stopping_method = method.into();
        self
    }

    pub fn with_handle_parsing_errors(mut self, val: bool) -> Self {
        self.handle_parsing_errors = val;
        self
    }

    pub fn with_return_intermediate_steps(mut self, val: bool) -> Self {
        self.return_intermediate_steps = val;
        self
    }

    pub fn with_verbose(mut self, val: bool) -> Self {
        self.verbose = val;
        self
    }

    pub fn with_callbacks(mut self, cb: CallbackManager) -> Self {
        self.callbacks = Some(cb);
        self
    }

    pub fn with_memory(mut self, m: Arc<dyn BaseMemory>) -> Self {
        self.memory = Some(m);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_metadata(mut self, meta: HashMap<String, Value>) -> Self {
        self.metadata = meta;
        self
    }

    fn tool_by_name(&self, name: &str) -> Option<Arc<dyn BaseTool>> {
        self.tools.iter().find(|t| t.name() == name).cloned()
    }

    pub async fn run(&self, inputs: &HashMap<String, Value>) -> Result<Value> {
        let mut resolved_inputs = inputs.clone();

        if let Some(ref memory) = self.memory {
            let memory_vars = memory.load_memory_variables(inputs).await?;
            resolved_inputs.extend(memory_vars);
        }

        if self.verbose {
            info!("AgentExecutor starting with inputs: {:?}", resolved_inputs);
        }

        if let Some(ref cb) = self.callbacks {
            let json = serde_json::to_value(&resolved_inputs).unwrap_or_default();
            cb.on_chain_start("AgentExecutor", &json);
        }

        let mut intermediate_steps: Vec<IntermediateStep> = Vec::new();

        let result = self.execute_loop(&resolved_inputs, &mut intermediate_steps).await;

        match result {
            Ok(mut finish) => {
                if let Some(ref cb) = self.callbacks {
                    let json = serde_json::to_value(&finish.return_values).unwrap_or_default();
                    cb.on_chain_end("AgentExecutor", &json);
                    let action_json = serde_json::to_value(&finish).unwrap_or_default();
                    cb.on_agent_finish(&action_json);
                }

                if let Some(ref memory) = self.memory {
                    memory.save_context(inputs, &finish.return_values).await?;
                }

                if self.return_intermediate_steps {
                    finish.return_values.insert(
                        "intermediate_steps".to_string(),
                        serde_json::to_value(&intermediate_steps).unwrap_or_default(),
                    );
                }

                let output = finish
                    .return_values
                    .get("output")
                    .cloned()
                    .or_else(|| finish.return_values.into_values().next())
                    .unwrap_or(Value::Null);

                Ok(output)
            }
            Err(e) => {
                if let Some(ref cb) = self.callbacks {
                    cb.on_chain_end("AgentExecutor", &Value::Null);
                }
                Err(e)
            }
        }
    }

    async fn execute_loop(
        &self,
        inputs: &HashMap<String, Value>,
        intermediate_steps: &mut Vec<IntermediateStep>,
    ) -> Result<AgentFinish> {
        for iteration in 0..self.max_iterations {
            if self.verbose {
                info!("Agent iteration {}/{}", iteration + 1, self.max_iterations);
            }

            let plan_result = if let Some(duration) = self.max_execution_time {
                match timeout(duration, self.agent.plan(intermediate_steps, inputs)).await {
                    Ok(result) => result,
                    Err(_) => {
                        warn!("Agent execution timed out after {:?}", duration);
                        return self
                            .agent
                            .return_stopped_response(
                                &self.early_stopping_method,
                                intermediate_steps,
                                inputs,
                            )
                            .await;
                    }
                }
            } else {
                self.agent.plan(intermediate_steps, inputs).await
            };

            match plan_result {
                Ok(AgentNextStep::Finish(finish)) => {
                    if self.verbose {
                        info!("Agent finished: {:?}", finish.return_values);
                    }
                    return Ok(finish);
                }
                Ok(AgentNextStep::Action(action)) => {
                    if self.verbose {
                        info!("Agent action: {} with input: {}", action.tool, action.tool_input);
                    }

                    if let Some(ref cb) = self.callbacks {
                        let action_json = serde_json::to_value(&action).unwrap_or_default();
                        cb.on_agent_action(&action_json);
                    }

                    let observation = match self.tool_by_name(&action.tool) {
                        Some(tool) => {
                            if let Some(ref cb) = self.callbacks {
                                let input_val = Value::String(action.tool_input.clone());
                                cb.on_tool_start(&tool.name(), &input_val);
                            }

                            let result = tool.invoke(&action.tool_input).await;

                            match result {
                                Ok(output) => {
                                    if let Some(ref cb) = self.callbacks {
                                        let output_val = Value::String(output.clone());
                                        cb.on_tool_end(&tool.name(), &output_val);
                                    }
                                    output
                                }
                                Err(e) => {
                                    let error_msg = format!("Tool '{}' error: {}", action.tool, e);
                                    if self.handle_parsing_errors {
                                        error_msg
                                    } else {
                                        return Err(e);
                                    }
                                }
                            }
                        }
                        None => {
                            let msg = format!("Tool '{}' not found. Available tools: {:?}",
                                action.tool,
                                self.tools.iter().map(|t| t.name()).collect::<Vec<_>>());
                            if self.handle_parsing_errors {
                                msg
                            } else {
                                return Err(ChainError::ToolError(msg));
                            }
                        }
                    };

                    let step = AgentStep::new(action, observation);
                    intermediate_steps.push(step);
                }
                Err(e) => {
                    if self.handle_parsing_errors {
                        warn!("Parsing error (handled): {}", e);
                        let fallback = AgentStep {
                            action: crate::types::AgentAction {
                                tool: String::new(),
                                tool_input: String::new(),
                                log: format!("Parsing error: {}", e),
                            },
                            observation: format!("Error: {}", e),
                        };
                        intermediate_steps.push(fallback);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        if self.verbose {
            warn!("Agent exceeded max iterations ({})", self.max_iterations);
        }

        self.agent
            .return_stopped_response(&self.early_stopping_method, intermediate_steps, inputs)
            .await
    }

    pub async fn stream(
        &self,
        inputs: &HashMap<String, Value>,
    ) -> Result<BoxStream<'static, Result<AgentStep>>> {
        let mut resolved_inputs = inputs.clone();

        if let Some(ref memory) = self.memory {
            let memory_vars = memory.load_memory_variables(inputs).await?;
            resolved_inputs.extend(memory_vars);
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<AgentStep>>(64);

        let agent = self.agent.clone();
        let tools = self.tools.clone();
        let max_iterations = self.max_iterations;
        let max_execution_time = self.max_execution_time;
        let early_stopping_method = self.early_stopping_method.clone();
        let handle_parsing_errors = self.handle_parsing_errors;
        let verbose = self.verbose;

        tokio::spawn(async move {
            let mut intermediate_steps: Vec<IntermediateStep> = Vec::new();

            for iteration in 0..max_iterations {
                if verbose {
                    info!("Stream iteration {}/{}", iteration + 1, max_iterations);
                }

                let plan_result = if let Some(duration) = max_execution_time {
                    match timeout(duration, agent.plan(&intermediate_steps, &resolved_inputs)).await {
                        Ok(result) => result,
                        Err(_) => {
                            let finish = agent
                                .return_stopped_response(&early_stopping_method, &intermediate_steps, &resolved_inputs)
                                .await;
                            match finish {
                                Ok(f) => {
                                    let _ = tx
                                        .send(Ok(AgentStep {
                                            action: crate::types::AgentAction {
                                                tool: "__finish__".into(),
                                                tool_input: String::new(),
                                                log: f.log.clone(),
                                            },
                                            observation: serde_json::to_string(&f.return_values)
                                                .unwrap_or_default(),
                                        }))
                                        .await;
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(e)).await;
                                }
                            }
                            break;
                        }
                    }
                } else {
                    agent.plan(&intermediate_steps, &resolved_inputs).await
                };

                match plan_result {
                    Ok(AgentNextStep::Finish(finish)) => {
                        let _ = tx
                            .send(Ok(AgentStep {
                                action: crate::types::AgentAction {
                                    tool: "__finish__".into(),
                                    tool_input: String::new(),
                                    log: finish.log.clone(),
                                },
                                observation: serde_json::to_string(&finish.return_values)
                                    .unwrap_or_default(),
                            }))
                            .await;
                        break;
                    }
                    Ok(AgentNextStep::Action(action)) => {
                        let observation = match tools.iter().find(|t| t.name() == action.tool) {
                            Some(tool) => match tool.invoke(&action.tool_input).await {
                                Ok(output) => output,
                                Err(e) => {
                                    if handle_parsing_errors {
                                        format!("Tool error: {}", e)
                                    } else {
                                        let _ = tx.send(Err(e)).await;
                                        break;
                                    }
                                }
                            },
                            None => {
                                let msg = format!("Tool '{}' not found", action.tool);
                                if handle_parsing_errors {
                                    msg
                                } else {
                                    let _ = tx
                                        .send(Err(ChainError::ToolError(msg)))
                                        .await;
                                    break;
                                }
                            }
                        };

                        let step = AgentStep::new(action.clone(), observation.clone());
                        let _ = tx.send(Ok(step)).await;
                        intermediate_steps.push(AgentStep::new(action, observation));
                    }
                    Err(e) => {
                        if handle_parsing_errors {
                            warn!("Stream parsing error (handled): {}", e);
                            let fallback = AgentStep {
                                action: crate::types::AgentAction {
                                    tool: String::new(),
                                    tool_input: String::new(),
                                    log: format!("Parsing error: {}", e),
                                },
                                observation: format!("Error: {}", e),
                            };
                            intermediate_steps.push(fallback.clone());
                            let _ = tx.send(Ok(fallback)).await;
                        } else {
                            let _ = tx.send(Err(e)).await;
                            break;
                        }
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx).boxed())
    }
}

impl Clone for AgentExecutor {
    fn clone(&self) -> Self {
        Self {
            agent: self.agent.clone(),
            tools: self.tools.clone(),
            max_iterations: self.max_iterations,
            max_execution_time: self.max_execution_time,
            early_stopping_method: self.early_stopping_method.clone(),
            handle_parsing_errors: self.handle_parsing_errors,
            return_intermediate_steps: self.return_intermediate_steps,
            verbose: self.verbose,
            callbacks: self.callbacks.clone(),
            memory: self.memory.clone(),
            tags: self.tags.clone(),
            metadata: self.metadata.clone(),
        }
    }
}
