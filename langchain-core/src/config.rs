//! Configuration for runnable execution.

use serde::{Serialize, Deserialize};

/// Configuration options passed to runnables during invocation.
///
/// Controls metadata, tracing, concurrency limits, and recursion depth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnableConfig {
    /// Tags attached to this run for filtering and tracing.
    pub tags: Vec<String>,
    /// Arbitrary metadata key-value pairs.
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Names of callback handlers to invoke.
    pub callbacks: Vec<String>,
    /// Maximum recursion depth for nested runnable calls (default: 25).
    pub recursion_limit: usize,
    /// Maximum number of concurrent operations (None = unlimited).
    pub max_concurrency: Option<usize>,
}

impl Default for RunnableConfig {
    fn default() -> Self {
        Self {
            tags: vec![],
            metadata: std::collections::HashMap::new(),
            callbacks: vec![],
            recursion_limit: 25,
            max_concurrency: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runnable_config_default() {
        let config = RunnableConfig::default();
        assert!(config.tags.is_empty());
        assert!(config.metadata.is_empty());
        assert!(config.callbacks.is_empty());
        assert_eq!(config.recursion_limit, 25);
        assert!(config.max_concurrency.is_none());
    }

    #[test]
    fn test_runnable_config_custom() {
        let config = RunnableConfig {
            tags: vec!["test".into()],
            metadata: {
                let mut m = std::collections::HashMap::new();
                m.insert("key".into(), serde_json::Value::String("val".into()));
                m
            },
            callbacks: vec!["handler1".into()],
            recursion_limit: 10,
            max_concurrency: Some(5),
        };
        assert_eq!(config.tags, vec!["test"]);
        assert_eq!(config.recursion_limit, 10);
        assert_eq!(config.max_concurrency, Some(5));
    }

    #[test]
    fn test_runnable_config_clone() {
        let config = RunnableConfig::default();
        let cloned = config.clone();
        assert_eq!(config.recursion_limit, cloned.recursion_limit);
    }

    #[test]
    fn test_runnable_config_debug() {
        let config = RunnableConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("recursion_limit"));
    }

    #[test]
    fn test_runnable_config_serde() {
        let config = RunnableConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RunnableConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.recursion_limit, 25);
    }

    #[test]
    fn test_runnable_config_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<RunnableConfig>();
        assert_sync::<RunnableConfig>();
    }

    #[test]
    fn test_runnable_config_max_concurrency() {
        let config = RunnableConfig {
            max_concurrency: Some(10),
            ..Default::default()
        };
        assert_eq!(config.max_concurrency, Some(10));
    }

    #[test]
    fn test_runnable_config_tags() {
        let config = RunnableConfig {
            tags: vec!["tag1".into(), "tag2".into()],
            ..Default::default()
        };
        assert_eq!(config.tags.len(), 2);
    }

    #[test]
    fn test_runnable_config_metadata() {
        let mut meta = std::collections::HashMap::new();
        meta.insert("key".into(), serde_json::Value::String("val".into()));
        let config = RunnableConfig {
            metadata: meta,
            ..Default::default()
        };
        assert_eq!(config.metadata.get("key").unwrap(), "val");
    }
}
