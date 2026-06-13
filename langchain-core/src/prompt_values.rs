//! Prompt value types for language model inputs.
//!
//! Provides [`PromptValue`] — an enum representing either a string prompt or
//! a chat prompt — along with [`StringPromptValue`] and [`ChatPromptValue`].
//! Prompt values can be converted to both LLM (pure text) inputs and chat
//! model inputs.

use crate::messages::{BaseMessage, MessageType};
use std::fmt;

/// A string prompt value wrapping a single text string.
///
/// When converted to messages, the text is wrapped in a
/// [`HumanMessage`].
#[derive(Debug, Clone)]
pub struct StringPromptValue {
    /// The prompt text.
    pub text: String,
}

impl StringPromptValue {
    /// Creates a new `StringPromptValue` from the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Returns the prompt as a plain string.
    pub fn to_string_value(&self) -> String {
        self.text.clone()
    }

    /// Returns the prompt as a list of [`BaseMessage`]s.
    ///
    /// The text is wrapped in a single [`HumanMessage`].
    pub fn to_messages(&self) -> Vec<BaseMessage> {
        vec![BaseMessage::new(&self.text, MessageType::Human)]
    }

    /// Returns the prompt as a list of [`BaseMessage`]s for chat models.
    ///
    /// Alias for [`StringPromptValue::to_messages`].
    pub fn to_chat_messages(&self) -> Vec<BaseMessage> {
        self.to_messages()
    }
}

impl fmt::Display for StringPromptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// A chat prompt value built from a list of messages.
///
/// When converted to a string, the messages are concatenated with role
/// prefixes (e.g. `Human: ...`).
#[derive(Debug, Clone)]
pub struct ChatPromptValue {
    /// The list of messages forming the prompt.
    pub messages: Vec<BaseMessage>,
}

impl ChatPromptValue {
    /// Creates a new `ChatPromptValue` from a list of messages.
    pub fn new(messages: Vec<BaseMessage>) -> Self {
        Self { messages }
    }

    /// Returns the prompt as a formatted string.
    ///
    /// Each message is rendered as `Role: Content`, joined by newlines.
    pub fn to_string_value(&self) -> String {
        self.messages
            .iter()
            .map(|msg| {
                let role = match msg.message_type {
                    MessageType::Human => "Human",
                    MessageType::AI => "AI",
                    MessageType::System => "System",
                    MessageType::Tool => "Tool",
                    MessageType::Function => "Function",
                    MessageType::Generic => "Generic",
                    MessageType::Chat => "Chat",
                };
                format!("{}: {}", role, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns the prompt as a list of [`BaseMessage`]s.
    pub fn to_messages(&self) -> Vec<BaseMessage> {
        self.messages.clone()
    }

    /// Returns the prompt as a list of [`BaseMessage`]s for chat models.
    ///
    /// Alias for [`ChatPromptValue::to_messages`].
    pub fn to_chat_messages(&self) -> Vec<BaseMessage> {
        self.to_messages()
    }
}

impl fmt::Display for ChatPromptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_value())
    }
}

/// A prompt value that can be either a string or a chat message list.
///
/// This enum unifies [`StringPromptValue`] and [`ChatPromptValue`] so that
/// downstream consumers (LLMs, chat models, chains) can handle either form
/// uniformly.
#[derive(Debug, Clone)]
pub enum PromptValue {
    /// A plain-text prompt value.
    String(StringPromptValue),
    /// A chat-oriented prompt value backed by a message list.
    Chat(ChatPromptValue),
}

impl PromptValue {
    /// Returns the prompt as a plain string.
    ///
    /// For the [`String`](PromptValue::String) variant this returns the
    /// inner text directly. For the [`Chat`](PromptValue::Chat) variant the
    /// messages are concatenated with role prefixes.
    pub fn to_string(&self) -> String {
        match self {
            PromptValue::String(sv) => sv.to_string_value(),
            PromptValue::Chat(cv) => cv.to_string_value(),
        }
    }

    /// Returns the prompt as a list of [`BaseMessage`]s.
    ///
    /// For the [`String`](PromptValue::String) variant the text is wrapped
    /// in a single [`HumanMessage`]. For the [`Chat`](PromptValue::Chat)
    /// variant the inner message list is returned.
    pub fn to_messages(&self) -> Vec<BaseMessage> {
        match self {
            PromptValue::String(sv) => sv.to_messages(),
            PromptValue::Chat(cv) => cv.to_messages(),
        }
    }

    /// Returns the prompt as a list of [`BaseMessage`]s for chat models.
    ///
    /// Alias for [`PromptValue::to_messages`].
    pub fn to_chat_messages(&self) -> Vec<BaseMessage> {
        self.to_messages()
    }
}

impl fmt::Display for PromptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<String> for PromptValue {
    fn from(text: String) -> Self {
        PromptValue::String(StringPromptValue::new(text))
    }
}

impl From<&str> for PromptValue {
    fn from(text: &str) -> Self {
        PromptValue::String(StringPromptValue::new(text))
    }
}

impl From<Vec<BaseMessage>> for PromptValue {
    fn from(messages: Vec<BaseMessage>) -> Self {
        PromptValue::Chat(ChatPromptValue::new(messages))
    }
}

impl From<StringPromptValue> for PromptValue {
    fn from(sv: StringPromptValue) -> Self {
        PromptValue::String(sv)
    }
}

impl From<ChatPromptValue> for PromptValue {
    fn from(cv: ChatPromptValue) -> Self {
        PromptValue::Chat(cv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_prompt_value_to_string() {
        let sv = StringPromptValue::new("Hello world");
        assert_eq!(sv.to_string_value(), "Hello world");
    }

    #[test]
    fn test_string_prompt_value_to_messages() {
        let sv = StringPromptValue::new("Hello");
        let msgs = sv.to_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "Hello");
        assert!(matches!(msgs[0].message_type, MessageType::Human));
    }

    #[test]
    fn test_chat_prompt_value_to_string() {
        let msgs = vec![
            BaseMessage::new("You are helpful.", MessageType::System),
            BaseMessage::new("Hi", MessageType::Human),
        ];
        let cv = ChatPromptValue::new(msgs);
        let s = cv.to_string_value();
        assert!(s.contains("System: You are helpful."));
        assert!(s.contains("Human: Hi"));
    }

    #[test]
    fn test_chat_prompt_value_to_messages() {
        let msgs = vec![
            BaseMessage::new("Hi", MessageType::Human),
            BaseMessage::new("Hello!", MessageType::AI),
        ];
        let cv = ChatPromptValue::new(msgs.clone());
        let result = cv.to_messages();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hi");
        assert_eq!(result[1].content, "Hello!");
    }

    #[test]
    fn test_prompt_value_from_string() {
        let pv: PromptValue = "Hello".into();
        assert_eq!(pv.to_string(), "Hello");
        let msgs = pv.to_messages();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_prompt_value_from_vec_messages() {
        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let pv: PromptValue = msgs.into();
        let result_msgs = pv.to_messages();
        assert_eq!(result_msgs.len(), 1);
        assert_eq!(result_msgs[0].content, "Hi");
    }

    #[test]
    fn test_prompt_value_display() {
        let pv = PromptValue::String(StringPromptValue::new("test"));
        assert_eq!(format!("{}", pv), "test");
    }

    #[test]
    fn test_string_prompt_value_display() {
        let sv = StringPromptValue::new("display test");
        assert_eq!(format!("{}", sv), "display test");
    }

    #[test]
    fn test_string_prompt_value_chat_messages() {
        let sv = StringPromptValue::new("chat");
        let msgs = sv.to_chat_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "chat");
    }

    #[test]
    fn test_chat_prompt_value_display() {
        let msgs = vec![BaseMessage::new("Hello", MessageType::Human)];
        let cv = ChatPromptValue::new(msgs);
        let s = format!("{}", cv);
        assert_eq!(s, "Human: Hello");
    }

    #[test]
    fn test_chat_prompt_value_chat_messages() {
        let msgs = vec![BaseMessage::new("Hi", MessageType::AI)];
        let cv = ChatPromptValue::new(msgs);
        let result = cv.to_chat_messages();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Hi");
    }

    #[test]
    fn test_prompt_value_from_string_prompt_value() {
        let sv = StringPromptValue::new("inner");
        let pv: PromptValue = sv.into();
        assert_eq!(pv.to_string(), "inner");
    }

    #[test]
    fn test_prompt_value_from_chat_prompt_value() {
        let msgs = vec![BaseMessage::new("msg", MessageType::Human)];
        let cv = ChatPromptValue::new(msgs);
        let pv: PromptValue = cv.into();
        let result = pv.to_messages();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_prompt_value_to_chat_messages() {
        let pv: PromptValue = "query".into();
        let msgs = pv.to_chat_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "query");
    }

    #[test]
    fn test_string_prompt_value_empty() {
        let sv = StringPromptValue::new("");
        assert_eq!(sv.to_string_value(), "");
        let msgs = sv.to_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "");
    }

    #[test]
    fn test_chat_prompt_value_all_message_types() {
        use MessageType::*;
        let types = vec![Human, AI, System, Tool, Function, Generic, Chat];
        let msgs: Vec<BaseMessage> = types
            .into_iter()
            .map(|t| BaseMessage::new("content", t))
            .collect();
        let cv = ChatPromptValue::new(msgs);
        let s = cv.to_string_value();
        assert!(s.contains("Human: content"));
        assert!(s.contains("AI: content"));
        assert!(s.contains("System: content"));
        assert!(s.contains("Tool: content"));
    }

    #[test]
    fn test_prompt_value_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<StringPromptValue>();
        assert_sync::<StringPromptValue>();
        assert_send::<ChatPromptValue>();
        assert_sync::<ChatPromptValue>();
        assert_send::<PromptValue>();
        assert_sync::<PromptValue>();
    }

    #[test]
    fn test_string_prompt_value_from_string() {
        let sv = StringPromptValue::new(String::from("owned"));
        assert_eq!(sv.to_string_value(), "owned");
    }

    #[test]
    fn test_string_prompt_value_display_trait() {
        let sv = StringPromptValue::new("display");
        assert_eq!(format!("{}", sv), "display");
    }

    #[test]
    fn test_chat_prompt_value_from_empty() {
        let cv = ChatPromptValue::new(vec![]);
        assert!(cv.to_messages().is_empty());
        assert_eq!(cv.to_string_value(), "");
    }

    #[test]
    fn test_prompt_value_string_variant_from_string() {
        let sv = StringPromptValue::new("test");
        let pv: PromptValue = sv.into();
        assert_eq!(pv.to_string(), "test");
    }

    #[test]
    fn test_prompt_value_chat_variant_from_messages() {
        let msgs = vec![BaseMessage::new("hi", MessageType::Human)];
        let pv: PromptValue = msgs.into();
        assert_eq!(pv.to_string(), "Human: hi");
    }

    #[test]
    fn test_prompt_value_from_str_ref() {
        let pv: PromptValue = "str ref".into();
        assert_eq!(pv.to_string(), "str ref");
    }

    #[test]
    fn test_chat_prompt_value_format_single_message() {
        let msgs = vec![
            BaseMessage::new("Hello", MessageType::Human),
        ];
        let cv = ChatPromptValue::new(msgs);
        assert_eq!(format!("{}", cv), "Human: Hello");
    }

    #[test]
    fn test_chat_prompt_value_format_multiple_messages() {
        let msgs = vec![
            BaseMessage::new("Hi", MessageType::Human),
            BaseMessage::new("Hello!", MessageType::AI),
        ];
        let cv = ChatPromptValue::new(msgs);
        let s = format!("{}", cv);
        assert_eq!(s, "Human: Hi\nAI: Hello!");
    }

    #[test]
    fn test_prompt_value_to_messages_string_variant() {
        let pv = PromptValue::String(StringPromptValue::new("text"));
        let msgs = pv.to_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "text");
        assert!(matches!(msgs[0].message_type, MessageType::Human));
    }
}
