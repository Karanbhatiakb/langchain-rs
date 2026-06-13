//! Prompt templating — string templates, chat message templates, and few-shot
//! templates.

use crate::errors::*;
use crate::messages::{BaseMessage, MessageType};
use regex::Regex;
use std::collections::HashMap;

/// A string prompt template with `{variable}` placeholders.
///
/// Extracts input variable names automatically from the template string.
///
/// # Example
///
/// ```ignore
/// let pt = PromptTemplate::from_template("Tell me a {adjective} story about {topic}.");
/// let result = pt.format(&HashMap::from([("adjective", "funny"), ("topic", "dogs")]));
/// ```
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// The raw template string with `{variable}` placeholders.
    pub template: String,
    /// List of variable names extracted from the template.
    pub input_variables: Vec<String>,
    /// Variables that are pre-filled (not required at format time).
    pub partial_variables: HashMap<String, String>,
    /// Whether to validate that all template variables are provided.
    pub validate_template: bool,
}

impl PromptTemplate {
    /// Creates a new `PromptTemplate` from a template string.
    ///
    /// Input variables are automatically extracted from `{variable}`
    /// placeholders.
    pub fn from_template(template: &str) -> Self {
        let re = Regex::new(r"\{(\w+)\}").unwrap();
        let input_variables: Vec<String> =
            re.captures_iter(template).map(|c| c[1].to_string()).collect();
        Self {
            template: template.to_string(),
            input_variables,
            partial_variables: HashMap::new(),
            validate_template: true,
        }
    }

    /// Sets partial variables that are pre-filled at construction time.
    pub fn with_partial(mut self, partial: HashMap<String, String>) -> Self {
        self.partial_variables = partial;
        self
    }

    /// Formats the template by substituting variables with the provided values.
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if a template variable is missing
    /// and `validate_template` is `true`.
    pub fn format(&self, kwargs: &HashMap<String, String>) -> Result<String> {
        let mut result = self.template.clone();
        let re = Regex::new(r"\{(\w+)\}").unwrap();
        for cap in re.captures_iter(&self.template.clone()) {
            let key = &cap[1];
            if let Some(val) = kwargs.get(key) {
                result = result.replace(&format!("{{{}}}", key), val);
            } else if let Some(val) = self.partial_variables.get(key) {
                result = result.replace(&format!("{{{}}}", key), val);
            } else if self.validate_template {
                return Err(ChainError::PromptError(format!("Missing variable: {}", key)));
            }
        }
        Ok(result)
    }

    /// Returns the list of input variable names.
    pub fn get_input_variables(&self) -> &[String] {
        &self.input_variables
    }

    /// Validates that all extracted variables are present in `input_variables`
    /// or `partial_variables`.
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if any variable is unaccounted for.
    pub fn validate_template(&self) -> Result<()> {
        let re = Regex::new(r"\{(\w+)\}").unwrap();
        for cap in re.captures_iter(&self.template) {
            let key = &cap[1];
            if !self.input_variables.contains(&key.to_string())
                && !self.partial_variables.contains_key(key)
            {
                return Err(ChainError::PromptError(format!(
                    "Variable {} not in input variables",
                    key
                )));
            }
        }
        Ok(())
    }
}

/// A single message template in a chat prompt.
#[derive(Debug, Clone)]
pub enum MessageTemplate {
    /// A system message template.
    System(String),
    /// A human/user message template.
    Human(String),
    /// An AI/assistant message template.
    AI(String),
    /// A placeholder that will be replaced with a list of messages at format
    /// time.
    Placeholder(String),
}

/// A chat-oriented prompt template composed of multiple [`MessageTemplate`]s.
///
/// # Example
///
/// ```ignore
/// let cpt = ChatPromptTemplate::new()
///     .add_system("You are a helpful assistant.")
///     .add_human("What is {topic}?");
/// let messages = cpt.format_prompt(&HashMap::from([("topic", "Rust")])).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ChatPromptTemplate {
    /// The ordered list of message templates.
    pub messages: Vec<MessageTemplate>,
}

impl ChatPromptTemplate {
    /// Creates an empty chat prompt template.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Appends a system message template (builder pattern).
    pub fn add_system(mut self, template: &str) -> Self {
        self.messages
            .push(MessageTemplate::System(template.to_string()));
        self
    }

    /// Appends a human message template (builder pattern).
    pub fn add_human(mut self, template: &str) -> Self {
        self.messages
            .push(MessageTemplate::Human(template.to_string()));
        self
    }

    /// Appends an AI message template (builder pattern).
    pub fn add_ai(mut self, template: &str) -> Self {
        self.messages.push(MessageTemplate::AI(template.to_string()));
        self
    }

    /// Appends a placeholder variable (builder pattern).
    ///
    /// At format time, the variable is expected to contain a JSON-serialized
    /// `Vec<BaseMessage>`.
    pub fn add_placeholder(mut self, variable_name: &str) -> Self {
        self.messages
            .push(MessageTemplate::Placeholder(variable_name.to_string()));
        self
    }

    /// Formats the chat prompt into a vector of [`BaseMessage`]s.
    ///
    /// Each message template is formatted with the provided `kwargs`.
    /// Placeholder variables are deserialized from JSON.
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if any template formatting fails.
    pub fn format_prompt(
        &self,
        kwargs: &HashMap<String, String>,
    ) -> Result<Vec<BaseMessage>> {
        let mut messages = Vec::new();
        for msg in &self.messages {
            match msg {
                MessageTemplate::System(template) => {
                    let pt = PromptTemplate::from_template(template);
                    let content = pt.format(kwargs)?;
                    messages.push(BaseMessage::new(content, MessageType::System));
                }
                MessageTemplate::Human(template) => {
                    let pt = PromptTemplate::from_template(template);
                    let content = pt.format(kwargs)?;
                    messages.push(BaseMessage::new(content, MessageType::Human));
                }
                MessageTemplate::AI(template) => {
                    let pt = PromptTemplate::from_template(template);
                    let content = pt.format(kwargs)?;
                    messages.push(BaseMessage::new(content, MessageType::AI));
                }
                MessageTemplate::Placeholder(variable_name) => {
                    if let Some(values_str) = kwargs.get(variable_name) {
                        if let Ok(parsed) =
                            serde_json::from_str::<Vec<BaseMessage>>(values_str)
                        {
                            messages.extend(parsed);
                        }
                    }
                }
            }
        }
        Ok(messages)
    }
}

impl Default for ChatPromptTemplate {
    fn default() -> Self {
        Self::new()
    }
}

/// A few-shot prompt template that interleaves examples between prefix and
/// suffix.
///
/// Each example is formatted with the `example_prompt` and joined by the
/// example separator.
#[derive(Debug, Clone)]
pub struct FewShotPromptTemplate {
    /// The list of example key-value maps.
    pub examples: Vec<HashMap<String, String>>,
    /// The prompt template used to format each example.
    pub example_prompt: PromptTemplate,
    /// Suffix after the examples.
    pub suffix: String,
    /// Prefix before the examples.
    pub prefix: String,
    /// Variables required for rendering the prefix/suffix.
    pub input_variables: Vec<String>,
    /// Separator between examples (default: `"\n\n"`).
    pub example_separator: String,
}

impl FewShotPromptTemplate {
    /// Creates a new few-shot prompt template.
    pub fn new(
        examples: Vec<HashMap<String, String>>,
        example_prompt: PromptTemplate,
        suffix: &str,
        prefix: &str,
        input_variables: Vec<String>,
    ) -> Self {
        Self {
            examples,
            example_prompt,
            suffix: suffix.to_string(),
            prefix: prefix.to_string(),
            input_variables,
            example_separator: "\n\n".into(),
        }
    }

    /// Sets the example separator string (builder pattern).
    pub fn with_example_separator(mut self, separator: &str) -> Self {
        self.example_separator = separator.to_string();
        self
    }

    /// Formats the full few-shot prompt, substituting variables in the prefix
    /// and suffix, interpolating formatted examples.
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if any template formatting fails.
    pub fn format(&self, kwargs: &HashMap<String, String>) -> Result<String> {
        let mut parts = Vec::new();

        if !self.prefix.is_empty() {
            let prefix_pt = PromptTemplate::from_template(&self.prefix);
            let formatted_prefix = prefix_pt.format(kwargs)?;
            parts.push(formatted_prefix);
        }

        let mut example_strings = Vec::new();
        for example in &self.examples {
            let formatted = self.example_prompt.format(example)?;
            example_strings.push(formatted);
        }
        parts.push(example_strings.join(&self.example_separator));

        if !self.suffix.is_empty() {
            let suffix_pt = PromptTemplate::from_template(&self.suffix);
            let formatted_suffix = suffix_pt.format(kwargs)?;
            parts.push(formatted_suffix);
        }

        Ok(parts.join(&self.example_separator))
    }
}

/// A prompt template for structured/typed input.
///
/// Validates that all keys declared as required in a JSON Schema are present
/// in the provided kwargs before delegating to the inner [`PromptTemplate`].
#[derive(Debug, Clone)]
pub struct StructuredPromptTemplate {
    /// A JSON Schema describing the expected input.
    pub schema: serde_json::Value,
    /// The underlying prompt template.
    pub template: PromptTemplate,
}

impl StructuredPromptTemplate {
    /// Creates a new `StructuredPromptTemplate` with the given schema and
    /// template.
    pub fn new(schema: serde_json::Value, template: PromptTemplate) -> Self {
        Self { schema, template }
    }

    /// Formats the template after validating that all keys listed in the
    /// schema's `required` array are present in `kwargs`.
    ///
    /// Only checks for key presence — does not perform full JSON Schema
    /// validation.
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if a required key is missing or
    /// if the schema does not contain a `required` array when expected.
    pub fn format(&self, kwargs: &HashMap<String, String>) -> Result<String> {
        if let Some(required) = self.schema.get("required").and_then(|r| r.as_array()) {
            for key in required {
                if let Some(key_str) = key.as_str() {
                    if !kwargs.contains_key(key_str) {
                        return Err(ChainError::PromptError(format!(
                            "Missing required key: {}",
                            key_str
                        )));
                    }
                }
            }
        }
        self.template.format(kwargs)
    }

    /// Returns a reference to the JSON Schema describing the expected input.
    pub fn get_schema(&self) -> &serde_json::Value {
        &self.schema
    }
}

/// Output of [`ImagePromptTemplate`], bundling a formatted prompt string with
/// image generation parameters.
#[derive(Debug, Clone)]
pub struct ImagePromptValue {
    /// The formatted prompt string.
    pub prompt: String,
    /// The image format (e.g. `"url"`, `"base64"`).
    pub image_format: String,
    /// The requested image size (e.g. `"512x512"`).
    pub size: Option<String>,
    /// The requested image quality (e.g. `"hd"`).
    pub quality: Option<String>,
}

impl ImagePromptValue {
    /// Creates a new `ImagePromptValue` with the given prompt string.
    ///
    /// `image_format` defaults to `"url"`, and `size`/`quality` default to
    /// `None`.
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            image_format: "url".to_string(),
            size: None,
            quality: None,
        }
    }
}

/// A prompt template for image generation.
///
/// Wraps a [`PromptTemplate`] and attaches image-generation parameters
/// (format, size, quality) to the resulting [`ImagePromptValue`].
#[derive(Debug, Clone)]
pub struct ImagePromptTemplate {
    /// The underlying string prompt template.
    pub template: PromptTemplate,
    /// The image format (e.g. `"url"`, `"base64"`).
    pub image_format: String,
    /// The requested image size (e.g. `"512x512"`).
    pub size: Option<String>,
    /// The requested image quality (e.g. `"hd"`).
    pub quality: Option<String>,
}

impl ImagePromptTemplate {
    /// Creates a new `ImagePromptTemplate` wrapping the given template.
    ///
    /// `image_format` defaults to `"url"`, and `size`/`quality` default to
    /// `None`.
    pub fn new(template: PromptTemplate) -> Self {
        Self {
            template,
            image_format: "url".to_string(),
            size: None,
            quality: None,
        }
    }

    /// Sets the image format (builder pattern).
    pub fn with_format(mut self, format: &str) -> Self {
        self.image_format = format.to_string();
        self
    }

    /// Sets the image size (builder pattern).
    pub fn with_size(mut self, size: &str) -> Self {
        self.size = Some(size.to_string());
        self
    }

    /// Sets the image quality (builder pattern).
    pub fn with_quality(mut self, quality: &str) -> Self {
        self.quality = Some(quality.to_string());
        self
    }

    /// Formats the template and wraps the result in an [`ImagePromptValue`].
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if the underlying template
    /// formatting fails.
    pub fn format(&self, kwargs: &HashMap<String, String>) -> Result<ImagePromptValue> {
        let prompt = self.template.format(kwargs)?;
        Ok(ImagePromptValue {
            prompt,
            image_format: self.image_format.clone(),
            size: self.size.clone(),
            quality: self.quality.clone(),
        })
    }
}

/// A few-shot prompt that uses [`ChatPromptTemplate`] for examples.
///
/// Unlike [`FewShotPromptTemplate`] (which produces a single string), this
/// type produces a `Vec<BaseMessage>` by formatting the prefix and suffix as
/// system messages and each example via the chat prompt template.
#[derive(Debug, Clone)]
pub struct FewShotPromptWithTemplates {
    /// The list of example key-value maps.
    pub examples: Vec<HashMap<String, String>>,
    /// The chat prompt template used to format each example.
    pub example_prompt: ChatPromptTemplate,
    /// Suffix after the examples (rendered as a system message).
    pub suffix: String,
    /// Prefix before the examples (rendered as a system message).
    pub prefix: String,
    /// Variables required for rendering the prefix/suffix.
    pub input_variables: Vec<String>,
    /// Separator between formatted example message groups.
    pub example_separator: String,
}

impl FewShotPromptWithTemplates {
    /// Creates a new `FewShotPromptWithTemplates`.
    pub fn new(
        examples: Vec<HashMap<String, String>>,
        example_prompt: ChatPromptTemplate,
        suffix: &str,
        prefix: &str,
        input_variables: Vec<String>,
    ) -> Self {
        Self {
            examples,
            example_prompt,
            suffix: suffix.to_string(),
            prefix: prefix.to_string(),
            input_variables,
            example_separator: "\n\n".to_string(),
        }
    }

    /// Formats the full few-shot chat prompt.
    ///
    /// The prefix and suffix are each rendered as a system message. Every
    /// example is formatted with the [`ChatPromptTemplate`], and the
    /// resulting message lists are concatenated.
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if any template formatting fails.
    pub fn format_prompt(
        &self,
        kwargs: &HashMap<String, String>,
    ) -> Result<Vec<BaseMessage>> {
        let mut messages = Vec::new();

        if !self.prefix.is_empty() {
            let prefix_pt = PromptTemplate::from_template(&self.prefix);
            let formatted_prefix = prefix_pt.format(kwargs)?;
            messages.push(BaseMessage::new(formatted_prefix, MessageType::System));
        }

        for example in &self.examples {
            let example_messages = self.example_prompt.format_prompt(example)?;
            messages.extend(example_messages);
        }

        if !self.suffix.is_empty() {
            let suffix_pt = PromptTemplate::from_template(&self.suffix);
            let formatted_suffix = suffix_pt.format(kwargs)?;
            messages.push(BaseMessage::new(formatted_suffix, MessageType::System));
        }

        Ok(messages)
    }
}

/// A specialized placeholder for chat message lists.
///
/// At format time, the placeholder looks up its `variable_name` in the
/// provided kwargs and parses the value as a JSON `Vec<BaseMessage>`. If the
/// variable is missing and `optional` is `true`, an empty vector is returned.
#[derive(Debug, Clone)]
pub struct MessagesPlaceholder {
    /// The key to look up in kwargs at format time.
    pub variable_name: String,
    /// Whether the variable is optional.
    pub optional: bool,
}

impl MessagesPlaceholder {
    /// Creates a new `MessagesPlaceholder` for the given variable name.
    ///
    /// `optional` defaults to `false`.
    pub fn new(variable_name: &str) -> Self {
        Self {
            variable_name: variable_name.to_string(),
            optional: false,
        }
    }

    /// Sets whether the placeholder is optional (builder pattern).
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Formats the placeholder by looking up `variable_name` in `kwargs` and
    /// parsing the value as a JSON `Vec<BaseMessage>`.
    ///
    /// If `optional` is `true` and the variable is missing, returns an empty
    /// vector.
    ///
    /// # Errors
    /// Returns [`ChainError::PromptError`] if the variable is missing and
    /// not optional, or if the value cannot be deserialized.
    pub fn format(&self, kwargs: &HashMap<String, String>) -> Result<Vec<BaseMessage>> {
        match kwargs.get(&self.variable_name) {
            Some(values_str) => serde_json::from_str::<Vec<BaseMessage>>(values_str)
                .map_err(|e| ChainError::PromptError(format!("Failed to parse messages: {}", e))),
            None => {
                if self.optional {
                    Ok(Vec::new())
                } else {
                    Err(ChainError::PromptError(format!(
                        "Missing required variable: {}",
                        self.variable_name
                    )))
                }
            }
        }
    }
}

/// A concrete chat prompt value (complement to
/// [`crate::messages::ChatPromptValue`]).
///
/// Carries a list of [`BaseMessage`]s along with a type discriminator.
#[derive(Debug, Clone)]
pub struct ChatPromptValueConcrete {
    /// The list of messages forming the prompt.
    pub messages: Vec<BaseMessage>,
    /// A type discriminator (e.g. `"chat"`).
    pub type_name: String,
}

impl ChatPromptValueConcrete {
    /// Creates a new `ChatPromptValueConcrete` from a list of messages.
    ///
    /// `type_name` defaults to `"chat"`.
    pub fn new(messages: Vec<BaseMessage>) -> Self {
        Self {
            messages,
            type_name: "chat".to_string(),
        }
    }

    /// Serializes the messages to a JSON string.
    ///
    /// # Errors
    /// Returns [`ChainError::SerializationError`] if serialization fails.
    pub fn to_string_value(&self) -> Result<String> {
        serde_json::to_string(&self.messages)
            .map_err(|e| ChainError::SerializationError(format!("{}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_prompt_template() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["topic"],
            "properties": {
                "topic": {"type": "string"}
            }
        });
        let template = PromptTemplate::from_template("Tell me about {topic}");
        let spt = StructuredPromptTemplate::new(schema, template);
        let kwargs = HashMap::from([("topic".to_string(), "Rust".to_string())]);
        let result = spt.format(&kwargs).unwrap();
        assert_eq!(result, "Tell me about Rust");
    }

    #[test]
    fn test_structured_prompt_template_missing_required() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["topic"],
            "properties": {
                "topic": {"type": "string"}
            }
        });
        let template = PromptTemplate::from_template("Tell me about {topic}");
        let spt = StructuredPromptTemplate::new(schema, template);
        let kwargs = HashMap::new();
        let result = spt.format(&kwargs);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_prompt_template() {
        let template = PromptTemplate::from_template("A {style} painting of {subject}");
        let ipt = ImagePromptTemplate::new(template)
            .with_format("base64")
            .with_size("512x512")
            .with_quality("hd");
        let kwargs = HashMap::from([
            ("style".to_string(), "realistic".to_string()),
            ("subject".to_string(), "mountains".to_string()),
        ]);
        let result = ipt.format(&kwargs).unwrap();
        assert_eq!(result.prompt, "A realistic painting of mountains");
        assert_eq!(result.image_format, "base64");
        assert_eq!(result.size.as_deref(), Some("512x512"));
        assert_eq!(result.quality.as_deref(), Some("hd"));
    }

    #[test]
    fn test_messages_placeholder() {
        let placeholder = MessagesPlaceholder::new("history");
        let msgs = vec![
            BaseMessage::new("Hello", MessageType::Human),
            BaseMessage::new("Hi there", MessageType::AI),
        ];
        let serialized = serde_json::to_string(&msgs).unwrap();
        let kwargs = HashMap::from([("history".to_string(), serialized)]);
        let result = placeholder.format(&kwargs).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hello");
        assert_eq!(result[1].content, "Hi there");
    }

    #[test]
    fn test_messages_placeholder_optional_missing() {
        let placeholder = MessagesPlaceholder::new("history").with_optional(true);
        let kwargs = HashMap::new();
        let result = placeholder.format(&kwargs).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_messages_placeholder_required_missing() {
        let placeholder = MessagesPlaceholder::new("history");
        let kwargs = HashMap::new();
        let result = placeholder.format(&kwargs);
        assert!(result.is_err());
    }

    #[test]
    fn test_chat_prompt_template_new() {
        let cpt = ChatPromptTemplate::new();
        assert!(cpt.messages.is_empty());
    }

    #[test]
    fn test_chat_prompt_template_with_system_and_human() {
        let cpt = ChatPromptTemplate::new()
            .add_system("You are helpful")
            .add_human("Tell me about {topic}");
        assert_eq!(cpt.messages.len(), 2);
        let kwargs = HashMap::from([("topic".into(), "Rust".into())]);
        let messages = cpt.format_prompt(&kwargs).unwrap();
        assert_eq!(messages.len(), 2);
        assert!(matches!(messages[0].message_type, MessageType::System));
        assert!(matches!(messages[1].message_type, MessageType::Human));
        assert_eq!(messages[1].content, "Tell me about Rust");
    }

    #[test]
    fn test_chat_prompt_template_with_ai() {
        let cpt = ChatPromptTemplate::new()
            .add_system("system")
            .add_human("human")
            .add_ai("ai");
        let kwargs = HashMap::new();
        let messages = cpt.format_prompt(&kwargs).unwrap();
        assert_eq!(messages.len(), 3);
        assert!(matches!(messages[2].message_type, MessageType::AI));
    }

    #[test]
    fn test_chat_prompt_template_with_placeholder() {
        let cpt = ChatPromptTemplate::new()
            .add_system("system")
            .add_placeholder("history")
            .add_human("human");
        let mut kwargs = HashMap::new();
        let history = vec![
            BaseMessage::new("prev human", MessageType::Human),
            BaseMessage::new("prev ai", MessageType::AI),
        ];
        kwargs.insert("history".into(), serde_json::to_string(&history).unwrap());
        let messages = cpt.format_prompt(&kwargs).unwrap();
        assert_eq!(messages.len(), 4);
    }

    #[test]
    fn test_chat_prompt_template_missing_variable() {
        let cpt = ChatPromptTemplate::new()
            .add_human("{unknown}");
        let kwargs = HashMap::new();
        let result = cpt.format_prompt(&kwargs);
        assert!(result.is_err());
    }

    #[test]
    fn test_chat_prompt_template_clone() {
        let cpt = ChatPromptTemplate::new()
            .add_system("s")
            .add_human("{q}");
        let cloned = cpt.clone();
        assert_eq!(cloned.messages.len(), 2);
    }

    #[test]
    fn test_prompt_template_default() {
        let pt = PromptTemplate::from_template("static text");
        assert!(pt.input_variables.is_empty());
        let kwargs = HashMap::new();
        let result = pt.format(&kwargs).unwrap();
        assert_eq!(result, "static text");
    }

    #[test]
    fn test_prompt_template_no_braces() {
        let pt = PromptTemplate::from_template("Hello world");
        let kwargs = HashMap::new();
        let result = pt.format(&kwargs).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_prompt_template_with_partial_variables() {
        let mut partials = HashMap::new();
        partials.insert("name".into(), "Bob".into());
        let pt = PromptTemplate::from_template("Hello {name}")
            .with_partial(partials);
        let kwargs = HashMap::new();
        let result = pt.format(&kwargs).unwrap();
        assert_eq!(result, "Hello Bob");
    }

    #[test]
    fn test_messages_placeholder_missing_with_optional() {
        let placeholder = MessagesPlaceholder::new("history").with_optional(true);
        let kwargs = HashMap::new();
        let result = placeholder.format(&kwargs).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_image_prompt_template_defaults() {
        let template = PromptTemplate::from_template("A {subject} image");
        let ipt = ImagePromptTemplate::new(template);
        let kwargs = HashMap::from([("subject".into(), "cat".into())]);
        let result = ipt.format(&kwargs).unwrap();
        assert_eq!(result.prompt, "A cat image");
        assert_eq!(result.image_format, "url");
        assert!(result.size.is_none());
        assert!(result.quality.is_none());
    }

    #[test]
    fn test_few_shot_prompt_with_templates() {
        let example_prompt = ChatPromptTemplate::new()
            .add_human("{input}")
            .add_ai("{output}");
        let examples = vec![
            HashMap::from([
                ("input".to_string(), "hi".to_string()),
                ("output".to_string(), "hello".to_string()),
            ]),
            HashMap::from([
                ("input".to_string(), "bye".to_string()),
                ("output".to_string(), "goodbye".to_string()),
            ]),
        ];
        let few_shot = FewShotPromptWithTemplates::new(
            examples,
            example_prompt,
            "Now answer: {question}",
            "You are helpful.",
            vec!["question".to_string()],
        );
        let kwargs = HashMap::from([("question".to_string(), "How are you?".to_string())]);
        let messages = few_shot.format_prompt(&kwargs).unwrap();
        assert!(messages.len() >= 4);
        let first_system = &messages[0];
        assert_eq!(first_system.content, "You are helpful.");
        assert!(matches!(first_system.message_type, MessageType::System));
        let last = messages.last().unwrap();
        assert!(last.content.contains("How are you?"));
    }
}
