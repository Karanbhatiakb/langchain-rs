//! Human-in-the-loop interrupt patterns for graph workflows.

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::errors::ChainError;
use std::sync::Arc;
use tokio::sync::watch;

use crate::nodes::Node;
use crate::state::StateSchema;

#[derive(Debug, Clone, PartialEq)]
pub enum InterruptPoint {
    Before,
    After,
}

#[derive(Debug, Clone)]
pub struct InterruptConfig {
    pub node_name: String,
    pub point: InterruptPoint,
    pub message: String,
}

impl InterruptConfig {
    pub fn before(node_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node_name: node_name.into(),
            point: InterruptPoint::Before,
            message: message.into(),
        }
    }

    pub fn after(node_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node_name: node_name.into(),
            point: InterruptPoint::After,
            message: message.into(),
        }
    }
}

pub struct HumanApprovalChannel {
    tx: watch::Sender<bool>,
    rx: watch::Receiver<bool>,
}

impl HumanApprovalChannel {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(false);
        Self { tx, rx }
    }

    pub fn approve(&self) {
        let _ = self.tx.send(true);
    }

    pub fn reject(&self) {
        let _ = self.tx.send(false);
    }

    pub async fn wait_for_approval(&mut self) -> bool {
        self.rx.changed().await.is_ok() && *self.rx.borrow()
    }
}

impl Default for HumanApprovalChannel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InterruptBeforeNode<S: StateSchema> {
    name: String,
    inner: Arc<dyn Node<S>>,
    channel: Arc<tokio::sync::Mutex<HumanApprovalChannel>>,
    message: String,
}

impl<S: StateSchema> InterruptBeforeNode<S> {
    pub fn new(
        name: impl Into<String>,
        inner: Arc<dyn Node<S>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            inner,
            channel: Arc::new(tokio::sync::Mutex::new(HumanApprovalChannel::new())),
            message: message.into(),
        }
    }

    pub fn approval_channel(&self) -> Arc<tokio::sync::Mutex<HumanApprovalChannel>> {
        self.channel.clone()
    }
}

#[async_trait]
impl<S: StateSchema> Node<S> for InterruptBeforeNode<S> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, state: S) -> Result<S> {
        let mut channel = self.channel.lock().await;
        let approved = channel.wait_for_approval().await;
        drop(channel);

        if !approved {
            return Err(ChainError::AgentError(format!(
                "Execution interrupted before node '{}': {}",
                self.name, self.message
            )));
        }

        self.inner.run(state).await
    }
}

pub struct InterruptAfterNode<S: StateSchema> {
    name: String,
    inner: Arc<dyn Node<S>>,
    channel: Arc<tokio::sync::Mutex<HumanApprovalChannel>>,
    message: String,
}

impl<S: StateSchema> InterruptAfterNode<S> {
    pub fn new(
        name: impl Into<String>,
        inner: Arc<dyn Node<S>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            inner,
            channel: Arc::new(tokio::sync::Mutex::new(HumanApprovalChannel::new())),
            message: message.into(),
        }
    }

    pub fn approval_channel(&self) -> Arc<tokio::sync::Mutex<HumanApprovalChannel>> {
        self.channel.clone()
    }
}

#[async_trait]
impl<S: StateSchema> Node<S> for InterruptAfterNode<S> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, state: S) -> Result<S> {
        let result = self.inner.run(state).await?;

        let mut channel = self.channel.lock().await;
        let approved = channel.wait_for_approval().await;
        drop(channel);

        if !approved {
            return Err(ChainError::AgentError(format!(
                "Execution interrupted after node '{}': {}",
                self.name, self.message
            )));
        }

        Ok(result)
    }
}

pub struct HumanInTheLoopConfig {
    pub interrupts: Vec<InterruptConfig>,
}

impl HumanInTheLoopConfig {
    pub fn new() -> Self {
        Self {
            interrupts: Vec::new(),
        }
    }

    pub fn interrupt_before(
        mut self,
        node_name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        self.interrupts.push(InterruptConfig::before(node_name, message));
        self
    }

    pub fn interrupt_after(
        mut self,
        node_name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        self.interrupts.push(InterruptConfig::after(node_name, message));
        self
    }

    pub fn has_interrupt_before(&self, node_name: &str) -> bool {
        self.interrupts.iter().any(|i| {
            i.node_name == node_name && i.point == InterruptPoint::Before
        })
    }

    pub fn has_interrupt_after(&self, node_name: &str) -> bool {
        self.interrupts.iter().any(|i| {
            i.node_name == node_name && i.point == InterruptPoint::After
        })
    }
}

impl Default for HumanInTheLoopConfig {
    fn default() -> Self {
        Self::new()
    }
}
