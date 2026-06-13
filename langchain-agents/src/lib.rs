//! Agent implementations for autonomous task execution.
//!
//! Provides the core agent trait in [`traits`], an [`executor`] for running
//! agents, a [`factory`] for creation, and strategies: ReAct, OpenAI
//! Functions, XML, Structured Chat, Tool Calling, JSON, Self-Ask,
//! Conversational, Plan-and-Execute, OpenAI Assistants, MRKL, and
//! Multi-Agent — each gated behind a feature flag.

pub mod types;
pub mod traits;
pub mod executor;
pub mod factory;
#[cfg(feature = "react")] pub mod react;
#[cfg(feature = "openai_functions")] pub mod openai_functions;
#[cfg(feature = "xml")] pub mod xml;
#[cfg(feature = "structured_chat")] pub mod structured_chat;
#[cfg(feature = "tool_calling")] pub mod tool_calling;
#[cfg(feature = "json")] pub mod json_agent;
#[cfg(feature = "self_ask")] pub mod self_ask;
#[cfg(feature = "conversational")] pub mod conversational;
#[cfg(feature = "plan_and_execute")] pub mod plan_and_execute;
#[cfg(feature = "openai_assistants")] pub mod openai_assistants;
#[cfg(feature = "mrkl")] pub mod mrkl;
#[cfg(feature = "multi_agent")] pub mod multi_agent;
pub mod toolkits;
