//! Graph edge types and condition routing.

use std::collections::HashMap;
use std::sync::Arc;

use crate::state::StateSchema;

pub const START: &str = "__start__";
pub const END: &str = "__end__";

#[derive(Clone)]
pub struct Edge<S: StateSchema> {
    pub from: String,
    pub to: String,
    pub _phantom: std::marker::PhantomData<S>,
}

impl<S: StateSchema> Edge<S> {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct ConditionalEdge<S: StateSchema> {
    pub from: String,
    pub condition_fn: Arc<dyn Fn(&S) -> String + Send + Sync>,
    pub mapping: HashMap<String, String>,
}

impl<S: StateSchema> ConditionalEdge<S> {
    pub fn new(
        from: impl Into<String>,
        condition_fn: Box<dyn Fn(&S) -> String + Send + Sync>,
        mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            from: from.into(),
            condition_fn: Arc::from(condition_fn),
            mapping,
        }
    }

    pub fn evaluate(&self, state: &S) -> String {
        let result = (self.condition_fn)(state);
        self.mapping
            .get(&result)
            .cloned()
            .unwrap_or_else(|| END.to_string())
    }

    pub fn clone_box(&self) -> ConditionalEdge<S> {
        ConditionalEdge {
            from: self.from.clone(),
            condition_fn: self.condition_fn.clone(),
            mapping: self.mapping.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AgentState;

    #[test]
    fn test_edge_new() {
        let edge = Edge::<AgentState>::new("a", "b");
        assert_eq!(edge.from, "a");
        assert_eq!(edge.to, "b");
    }

    #[test]
    fn test_edge_with_start_end() {
        let edge = Edge::<AgentState>::new(START, END);
        assert_eq!(edge.from, START);
        assert_eq!(edge.to, END);
    }

    #[test]
    fn test_conditional_edge_new() {
        let mapping = HashMap::from([
            ("yes".into(), "process".into()),
            ("no".into(), END.into()),
        ]);
        let cond = ConditionalEdge::<AgentState>::new(
            "check",
            Box::new(|_: &AgentState| "yes".into()),
            mapping,
        );
        assert_eq!(cond.from, "check");
    }

    #[test]
    fn test_conditional_edge_evaluate() {
        let mapping = HashMap::from([
            ("yes".into(), "process".into()),
            ("no".into(), END.into()),
        ]);
        let cond = ConditionalEdge::<AgentState>::new(
            "check",
            Box::new(|_: &AgentState| "yes".into()),
            mapping,
        );
        assert_eq!(cond.evaluate(&AgentState::new(vec![])), "process");
    }

    #[test]
    fn test_conditional_edge_evaluate_fallback() {
        let mapping = HashMap::from([
            ("yes".into(), "process".into()),
        ]);
        let cond = ConditionalEdge::<AgentState>::new(
            "check",
            Box::new(|_: &AgentState| "unknown".into()),
            mapping,
        );
        assert_eq!(cond.evaluate(&AgentState::new(vec![])), END);
    }

    #[test]
    fn test_conditional_edge_clone() {
        let mapping = HashMap::from([("a".into(), "b".into())]);
        let cond = ConditionalEdge::<AgentState>::new(
            "from",
            Box::new(|_: &AgentState| "a".into()),
            mapping,
        );
        let cloned = cond.clone_box();
        assert_eq!(cloned.from, "from");
        assert_eq!(cloned.evaluate(&AgentState::new(vec![])), "b");
    }

    #[test]
    fn test_edge_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Edge<AgentState>>();
        assert_sync::<Edge<AgentState>>();
    }
}
