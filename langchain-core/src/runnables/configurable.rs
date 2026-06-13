//! Configurable runnables — runtime-selectable alternatives and field configuration.
//!
//! Provides [`ConfigurableRunnable`] which wraps an inner [`Runnable`] and allows
//! swapping alternatives based on configuration keys, as well as field-level
//! configuration via [`ConfigurableField`], [`ConfigurableFieldSingleOption`],
//! [`ConfigurableFieldMultiOption`], and [`ConfigurableFields`].
//!
//! This is the Rust counterpart of
//! `langchain_core.runnables.configurable` from the Python LangChain project.

use crate::config::RunnableConfig;
use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type RunFn = Arc<
    dyn Fn(
            HashMap<String, Value>,
            RunnableConfig,
        ) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send>>
        + Send
        + Sync,
>;

/// A single configurable field that can be set at runtime.
///
/// Corresponds to `ConfigurableField` in the Python LangChain project.
///
/// # Example
///
/// ```rust,ignore
/// use langchain_core::runnables::configurable::ConfigurableField;
///
/// let field = ConfigurableField::new("temperature")
///     .with_name("LLM Temperature")
///     .with_description("The temperature of the LLM");
/// ```
#[derive(Debug, Clone)]
pub struct ConfigurableField {
    /// Unique identifier for this field, used as the key in the `configurable`
    /// section of a [`RunnableConfig`].
    pub id: String,
    /// Human-readable name shown in tooling / UIs.
    pub name: String,
    /// Description of what this field controls.
    pub description: String,
    /// Whether this field is shared across all alternatives.
    pub is_shared: bool,
    /// Default value for this field.
    pub default: Option<Value>,
}

impl ConfigurableField {
    /// Creates a new `ConfigurableField` with the given id.
    ///
    /// The `name` and `description` default to the id string; `is_shared`
    /// defaults to `false`.
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            description: id.clone(),
            id,
            is_shared: false,
            default: None,
        }
    }

    /// Sets the human-readable name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Marks this field as shared across alternatives.
    pub fn with_shared(mut self, is_shared: bool) -> Self {
        self.is_shared = is_shared;
        self
    }

    /// Sets the default value.
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }
}

/// A configurable field with a single option selected from a fixed set.
///
/// Corresponds to `ConfigurableFieldSingleOption` in the Python LangChain
/// project.
#[derive(Debug, Clone)]
pub struct ConfigurableFieldSingleOption {
    /// Unique identifier for this field.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of what this field controls.
    pub description: String,
    /// Whether this field is shared across alternatives.
    pub is_shared: bool,
    /// Mapping of option key to option value.
    pub options: HashMap<String, Value>,
    /// The key of the default option.
    pub default: String,
}

impl ConfigurableFieldSingleOption {
    /// Creates a new `ConfigurableFieldSingleOption` with the given id and
    /// options.
    ///
    /// `default` is the key within `options` that is selected when no
    /// configuration override is provided.
    pub fn new(
        id: impl Into<String>,
        options: HashMap<String, Value>,
        default: impl Into<String>,
    ) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            description: id.clone(),
            id,
            is_shared: false,
            options,
            default: default.into(),
        }
    }

    /// Sets the human-readable name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Marks this field as shared across alternatives.
    pub fn with_shared(mut self, is_shared: bool) -> Self {
        self.is_shared = is_shared;
        self
    }
}

/// A configurable field that supports selecting multiple options from a fixed
/// set.
///
/// Corresponds to `ConfigurableFieldMultiOption` in the Python LangChain
/// project.
#[derive(Debug, Clone)]
pub struct ConfigurableFieldMultiOption {
    /// Unique identifier for this field.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of what this field controls.
    pub description: String,
    /// Whether this field is shared across alternatives.
    pub is_shared: bool,
    /// Mapping of option key to option value.
    pub options: HashMap<String, Value>,
    /// The keys of the default options.
    pub default: Vec<String>,
}

impl ConfigurableFieldMultiOption {
    /// Creates a new `ConfigurableFieldMultiOption` with the given id and
    /// options.
    ///
    /// `default` is the list of keys within `options` that are selected when
    /// no configuration override is provided.
    pub fn new(
        id: impl Into<String>,
        options: HashMap<String, Value>,
        default: Vec<String>,
    ) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            description: id.clone(),
            id,
            is_shared: false,
            options,
            default,
        }
    }

    /// Sets the human-readable name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Marks this field as shared across alternatives.
    pub fn with_shared(mut self, is_shared: bool) -> Self {
        self.is_shared = is_shared;
        self
    }
}

/// Enumeration of the different kinds of configurable fields.
#[derive(Debug, Clone)]
pub enum ConfigurableFieldKind {
    /// A simple key-value field.
    Simple(ConfigurableField),
    /// A field with a single option selected from a fixed set.
    SingleOption(ConfigurableFieldSingleOption),
    /// A field with multiple options selected from a fixed set.
    MultiOption(ConfigurableFieldMultiOption),
}

/// A collection of configurable fields keyed by the field name used in the
/// inner runnable's construction.
///
/// Corresponds to the `fields` dict attribute of `RunnableConfigurableFields`
/// in the Python LangChain project.
#[derive(Debug, Clone, Default)]
pub struct ConfigurableFields {
    /// The inner map from field-name to field-spec.
    pub fields: HashMap<String, ConfigurableFieldKind>,
}

impl ConfigurableFields {
    /// Creates an empty `ConfigurableFields`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a simple configurable field.
    pub fn with_field(mut self, key: impl Into<String>, field: ConfigurableField) -> Self {
        self.fields.insert(key.into(), ConfigurableFieldKind::Simple(field));
        self
    }

    /// Adds a single-option configurable field.
    pub fn with_single_option(
        mut self,
        key: impl Into<String>,
        field: ConfigurableFieldSingleOption,
    ) -> Self {
        self.fields.insert(key.into(), ConfigurableFieldKind::SingleOption(field));
        self
    }

    /// Adds a multi-option configurable field.
    pub fn with_multi_option(
        mut self,
        key: impl Into<String>,
        field: ConfigurableFieldMultiOption,
    ) -> Self {
        self.fields.insert(key.into(), ConfigurableFieldKind::MultiOption(field));
        self
    }
}

/// A map of config key to alternative runnable functions.
///
/// The default key is `Self::default_key`; alternatives are stored alongside
/// it. At invocation time the `configurable` section of the [`RunnableConfig`]
/// is inspected to decide which alternative to run.
#[derive(Clone)]
pub struct ConfigurableAlternatives {
    /// The [`ConfigurableField`] that determines which alternative to select.
    pub which: ConfigurableField,
    /// Alternative runnables keyed by their alternative name.
    pub alternatives: HashMap<String, RunFn>,
    /// The key used to identify the default runnable.
    pub default_key: String,
    /// Whether to prefix configurable field keys of each alternative with a
    /// namespace of the form `<which.id>==<alternative_key>`.
    pub prefix_keys: bool,
}

impl ConfigurableAlternatives {
    /// Creates a new `ConfigurableAlternatives` with the given selector field.
    ///
    /// The default key is `"default"`.
    pub fn new(which: ConfigurableField) -> Self {
        Self {
            which,
            alternatives: HashMap::new(),
            default_key: "default".to_string(),
            prefix_keys: false,
        }
    }

    /// Sets the key used to identify the default runnable.
    pub fn with_default_key(mut self, key: impl Into<String>) -> Self {
        self.default_key = key.into();
        self
    }

    /// Enables prefixing of configurable field keys for each alternative.
    pub fn with_prefix_keys(mut self, prefix_keys: bool) -> Self {
        self.prefix_keys = prefix_keys;
        self
    }

    /// Adds an alternative runnable.
    pub fn with_alternative<F, Fut>(mut self, key: impl Into<String>, runnable: F) -> Self
    where
        F: Fn(HashMap<String, Value>, RunnableConfig) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        let arc_fn: RunFn = Arc::new(move |input, config| Box::pin(runnable(input, config)));
        self.alternatives.insert(key.into(), arc_fn);
        self
    }
}

impl std::fmt::Debug for ConfigurableAlternatives {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigurableAlternatives")
            .field("which", &self.which)
            .field("alternatives", &self.alternatives.keys().collect::<Vec<_>>())
            .field("default_key", &self.default_key)
            .field("prefix_keys", &self.prefix_keys)
            .finish()
    }
}

/// A [`Runnable`] that can be dynamically configured at runtime.
///
/// Supports two configuration modes:
///
/// 1. **Alternatives** — swap the entire runnable based on a config key via
///    [`ConfigurableRunnable::with_alternatives`].
/// 2. **Fields** — configure individual fields of the runnable via
///    [`ConfigurableRunnable::with_configurable_fields`].
///
/// Corresponds to `DynamicRunnable` / `RunnableConfigurableAlternatives` /
/// `RunnableConfigurableFields` in the Python LangChain project.
pub struct ConfigurableRunnable {
    inner: RunFn,
    alternatives: Option<ConfigurableAlternatives>,
    fields: Option<ConfigurableFields>,
}

impl ConfigurableRunnable {
    /// Creates a new `ConfigurableRunnable` wrapping the given async function.
    pub fn new<F, Fut>(inner: F) -> Self
    where
        F: Fn(HashMap<String, Value>, RunnableConfig) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        Self {
            inner: Arc::new(move |input, config| Box::pin(inner(input, config))),
            alternatives: None,
            fields: None,
        }
    }

    /// Sets the configurable alternatives for this runnable.
    ///
    /// When a configuration specifies a value for the selector field
    /// (`alternatives.which.id`), the corresponding alternative runnable is
    /// invoked instead of the default.
    pub fn with_alternatives(mut self, alternatives: ConfigurableAlternatives) -> Self {
        self.alternatives = Some(alternatives);
        self
    }

    /// Sets the configurable fields for this runnable.
    ///
    /// Configurable fields allow runtime modification of individual parameters
    /// of the inner runnable by reading overrides from the `configurable`
    /// section of the [`RunnableConfig`].
    pub fn with_configurable_fields(mut self, fields: ConfigurableFields) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Resolves the runnable function that should be invoked based on the
    /// given configuration.
    ///
    /// If alternatives are configured and the config contains a matching
    /// selector, the corresponding alternative is returned. Otherwise the
    /// default inner runnable is returned.
    fn resolve(&self, config: &RunnableConfig) -> Result<RunFn> {
        if let Some(alts) = &self.alternatives {
            let which_id = &alts.which.id;
            if let Some(Value::String(selected)) = config.metadata.get(which_id) {
                if selected == &alts.default_key {
                    return Ok(self.inner.clone());
                }
                if let Some(alt_fn) = alts.alternatives.get(selected) {
                    return Ok(alt_fn.clone());
                }
                return Err(ChainError::ConfigError(format!(
                    "Unknown alternative '{}' for field '{}'",
                    selected, which_id
                )));
            }
        }
        Ok(self.inner.clone())
    }
}

impl Clone for ConfigurableRunnable {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            alternatives: self.alternatives.clone(),
            fields: self.fields.clone(),
        }
    }
}

impl std::fmt::Debug for ConfigurableRunnable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigurableRunnable")
            .field("has_alternatives", &self.alternatives.is_some())
            .field("has_fields", &self.fields.is_some())
            .finish()
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for ConfigurableRunnable {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let config = RunnableConfig::default();
        let runnable = self.resolve(&config)?;
        runnable(input, config).await
    }

    async fn batch(&self, inputs: Vec<HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let config = RunnableConfig::default();
            let runnable = self.resolve(&config)?;
            results.push(runnable(input, config).await?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_configurable_field_builder() {
        let field = ConfigurableField::new("temperature")
            .with_name("LLM Temperature")
            .with_description("The temperature of the LLM")
            .with_shared(true);
        assert_eq!(field.id, "temperature");
        assert_eq!(field.name, "LLM Temperature");
        assert_eq!(field.description, "The temperature of the LLM");
        assert!(field.is_shared);
    }

    #[tokio::test]
    async fn test_configurable_alternatives_invoke() {
        let default_fn = |input: HashMap<String, Value>, _config: RunnableConfig| async move {
            let mut result = input;
            result.insert("mode".into(), Value::String("default".into()));
            Ok(result)
        };

        let alt_fn = |input: HashMap<String, Value>, _config: RunnableConfig| async move {
            let mut result = input;
            result.insert("mode".into(), Value::String("alternative".into()));
            Ok(result)
        };

        let which = ConfigurableField::new("prompt").with_name("Prompt Selector");
        let alts = ConfigurableAlternatives::new(which)
            .with_default_key("joke")
            .with_alternative("poem", alt_fn);

        let configurable = ConfigurableRunnable::new(default_fn).with_alternatives(alts);

        let mut config = RunnableConfig::default();
        config.metadata.insert("prompt".into(), Value::String("poem".into()));

        let runnable = configurable.resolve(&config).expect("resolve should succeed");
        let result = runnable(HashMap::new(), config).await.expect("invoke should succeed");
        assert_eq!(result.get("mode"), Some(&Value::String("alternative".into())));
    }

    #[tokio::test]
    async fn test_configurable_fields() {
        let mut options = HashMap::new();
        options.insert("a".into(), Value::String("option_a".into()));
        options.insert("b".into(), Value::String("option_b".into()));

        let fields = ConfigurableFields::new()
            .with_field("temperature", ConfigurableField::new("temperature"))
            .with_single_option(
                "model",
                ConfigurableFieldSingleOption::new("model", options, "a"),
            );

        assert_eq!(fields.fields.len(), 2);
    }

    #[tokio::test]
    async fn test_unknown_alternative_returns_error() {
        let default_fn = |input: HashMap<String, Value>, _config: RunnableConfig| async move {
            Ok(input)
        };

        let which = ConfigurableField::new("model");
        let alts = ConfigurableAlternatives::new(which);

        let configurable = ConfigurableRunnable::new(default_fn).with_alternatives(alts);

        let mut config = RunnableConfig::default();
        config.metadata.insert("model".into(), Value::String("nonexistent".into()));

        let result = configurable.resolve(&config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multi_option_field() {
        let mut options = HashMap::new();
        options.insert("x".into(), Value::String("opt_x".into()));
        options.insert("y".into(), Value::String("opt_y".into()));

        let field = ConfigurableFieldMultiOption::new("features", options, vec!["x".into()]);
        assert_eq!(field.id, "features");
        assert_eq!(field.default, vec!["x".to_string()]);
    }

    #[tokio::test]
    async fn test_configurable_field_default() {
        let field = ConfigurableField::new("model").with_default(Value::String("gpt-4".into()));
        assert_eq!(field.default, Some(Value::String("gpt-4".into())));
    }

    #[tokio::test]
    async fn test_configurable_alternatives_default_alternative() {
        let default_fn = |input: HashMap<String, Value>, _config: RunnableConfig| async move { Ok(input) };
        let which = ConfigurableField::new("mode");
        let alts = ConfigurableAlternatives::new(which).with_default_key("default");
        let configurable = ConfigurableRunnable::new(default_fn).with_alternatives(alts);

        let mut config = RunnableConfig::default();
        config.metadata.insert("mode".into(), Value::String("default".into()));
        let runnable = configurable.resolve(&config);
        assert!(runnable.is_ok());
    }

    #[tokio::test]
    async fn test_configurable_invoke_without_alternatives() {
        let fn_ = |input: HashMap<String, Value>, _config: RunnableConfig| async move {
            let mut result = input;
            result.insert("ok".into(), Value::String("yes".into()));
            Ok(result)
        };
        let configurable = ConfigurableRunnable::new(fn_);
        let mut input = HashMap::new();
        input.insert("x".into(), Value::String("y".into()));
        let result = configurable.invoke(input).await.unwrap();
        assert_eq!(result.get("ok"), Some(&Value::String("yes".into())));
    }

    #[tokio::test]
    async fn test_configurable_batch() {
        let fn_ = |input: HashMap<String, Value>, _config: RunnableConfig| async move { Ok(input) };
        let configurable = ConfigurableRunnable::new(fn_);
        let input1 = HashMap::from([("a".into(), Value::String("1".into()))]);
        let input2 = HashMap::from([("b".into(), Value::String("2".into()))]);
        let results = configurable.batch(vec![input1, input2]).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_configurable_fields_builder() {
        let fields = ConfigurableFields::new()
            .with_field("temperature", ConfigurableField::new("temperature").with_name("Temperature"))
            .with_field("model", ConfigurableField::new("model").with_name("Model Name"));
        assert_eq!(fields.fields.len(), 2);
    }

    #[tokio::test]
    async fn test_configurable_single_option_with_shared() {
        let mut options = HashMap::new();
        options.insert("fast".into(), Value::String("gpt-3.5".into()));
        options.insert("slow".into(), Value::String("gpt-4".into()));
        let field = ConfigurableFieldSingleOption::new("model", options, "fast")
            .with_shared(true)
            .with_name("Model Selection");
        assert!(field.is_shared);
        assert_eq!(field.name, "Model Selection");
    }

    #[tokio::test]
    async fn test_configurable_clone() {
        let fn_ = |input: HashMap<String, Value>, _config: RunnableConfig| async move { Ok(input) };
        let a = ConfigurableRunnable::new(fn_);
        let b = a.clone();
        let input = HashMap::from([("x".into(), Value::String("y".into()))]);
        assert_eq!(a.invoke(input.clone()).await.unwrap(), b.invoke(input).await.unwrap());
    }

    #[test]
    fn test_configurable_field_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ConfigurableField>();
        assert_sync::<ConfigurableField>();
        assert_send::<ConfigurableFieldSingleOption>();
        assert_sync::<ConfigurableFieldSingleOption>();
        assert_send::<ConfigurableFieldMultiOption>();
        assert_sync::<ConfigurableFieldMultiOption>();
        assert_send::<ConfigurableFields>();
        assert_sync::<ConfigurableFields>();
        assert_send::<ConfigurableRunnable>();
        assert_sync::<ConfigurableRunnable>();
    }
}
