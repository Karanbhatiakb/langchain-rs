//! Streaming primitives — chunk types, stream events, and channel-based
//! streaming infrastructure.

use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

/// A chunk of raw bytes produced during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteChunk {
    /// The binary data for this chunk.
    pub data: Vec<u8>,
    /// Optional metadata associated with the chunk.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ByteChunk {
    /// Creates a new `ByteChunk` with the given data.
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
        }
    }

    /// Sets metadata and returns `self` (builder pattern).
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// A structured event in a stream's lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    /// The stream has started for a named runnable with the given input.
    Start { name: String, input: serde_json::Value },
    /// A single chunk of output.
    Chunk { chunk: serde_json::Value },
    /// The stream has ended for a named runnable with the final output.
    End { name: String, output: serde_json::Value },
    /// An error occurred during streaming.
    Error { error: String },
}

/// Manages a channel-based byte stream, allowing producers to push chunks
/// and errors to a single receiver.
pub struct StreamManager {
    #[allow(dead_code)]
    buffer_size: usize,
    sender: Option<mpsc::UnboundedSender<Result<ByteChunk>>>,
}

impl StreamManager {
    /// Creates a new `StreamManager` with the configured buffer size.
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            sender: None,
        }
    }

    /// Initialises the channel and returns the receiver end.
    pub fn channel(&mut self) -> mpsc::UnboundedReceiver<Result<ByteChunk>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.sender = Some(tx);
        rx
    }

    /// Pushes a chunk into the channel.
    ///
    /// # Errors
    /// Returns [`ChainError::StreamError`] if the channel is not configured
    /// or has been closed.
    pub fn push(&self, chunk: ByteChunk) -> Result<()> {
        if let Some(ref sender) = self.sender {
            sender.send(Ok(chunk)).map_err(|_| {
                ChainError::StreamError("Failed to push chunk: channel closed".into())
            })
        } else {
            Err(ChainError::StreamError("No channel configured".into()))
        }
    }

    /// Pushes an error into the channel.
    ///
    /// # Errors
    /// Returns [`ChainError::StreamError`] if the channel is not configured
    /// or has been closed.
    pub fn push_error(&self, error: ChainError) -> Result<()> {
        if let Some(ref sender) = self.sender {
            sender.send(Err(error)).map_err(|_| {
                ChainError::StreamError("Failed to push error: channel closed".into())
            })
        } else {
            Err(ChainError::StreamError("No channel configured".into()))
        }
    }

    /// Closes the channel by dropping the sender.
    pub fn close(&self) {
        if let Some(ref sender) = self.sender {
            drop(sender.clone());
        }
    }
}

/// A [`futures::stream::Stream`] wrapping an `mpsc::UnboundedReceiver` of byte
/// chunks.
pub struct ByteStream {
    receiver: mpsc::UnboundedReceiver<Result<ByteChunk>>,
}

impl ByteStream {
    /// Wraps an existing receiver into a `ByteStream`.
    pub fn new(receiver: mpsc::UnboundedReceiver<Result<ByteChunk>>) -> Self {
        Self { receiver }
    }
}

impl futures::stream::Stream for ByteStream {
    type Item = Result<ByteChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

/// A pinned, boxed, sendable byte-stream.
pub type BoxBytesStream = Pin<Box<dyn futures::stream::Stream<Item = Result<ByteChunk>> + Send>>;

/// Converts a `Vec<Result<T>>` into a pinned, boxed stream.
pub fn into_stream<T: Send + 'static>(
    items: Vec<Result<T>>,
) -> Pin<Box<dyn futures::stream::Stream<Item = Result<T>> + Send>> {
    Box::pin(futures::stream::iter(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[test]
    fn test_byte_chunk_new() {
        let chunk = ByteChunk::new(vec![1, 2, 3]);
        assert_eq!(chunk.data, vec![1, 2, 3]);
        assert!(chunk.metadata.is_empty());
    }

    #[test]
    fn test_byte_chunk_empty() {
        let chunk = ByteChunk::new(vec![]);
        assert!(chunk.data.is_empty());
    }

    #[test]
    fn test_byte_chunk_with_metadata() {
        let mut meta = HashMap::new();
        meta.insert("key".into(), serde_json::Value::String("val".into()));
        let chunk = ByteChunk::new(vec![0]).with_metadata(meta);
        assert_eq!(chunk.metadata.get("key").unwrap(), "val");
    }

    #[test]
    fn test_byte_chunk_clone() {
        let chunk = ByteChunk::new(vec![1, 2, 3]);
        let cloned = chunk.clone();
        assert_eq!(cloned.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_stream_event_start() {
        let event = StreamEvent::Start {
            name: "chain".into(),
            input: serde_json::json!({"x": 1}),
        };
        match event {
            StreamEvent::Start { name, input } => {
                assert_eq!(name, "chain");
                assert_eq!(input["x"], 1);
            }
            _ => panic!("expected Start"),
        }
    }

    #[test]
    fn test_stream_event_chunk() {
        let event = StreamEvent::Chunk {
            chunk: serde_json::json!({"token": "hello"}),
        };
        match event {
            StreamEvent::Chunk { chunk } => {
                assert_eq!(chunk["token"], "hello");
            }
            _ => panic!("expected Chunk"),
        }
    }

    #[test]
    fn test_stream_event_end() {
        let event = StreamEvent::End {
            name: "chain".into(),
            output: serde_json::json!("done"),
        };
        match event {
            StreamEvent::End { name, output } => {
                assert_eq!(name, "chain");
                assert_eq!(output, "done");
            }
            _ => panic!("expected End"),
        }
    }

    #[test]
    fn test_stream_event_error() {
        let event = StreamEvent::Error {
            error: "something went wrong".into(),
        };
        match event {
            StreamEvent::Error { error } => {
                assert_eq!(error, "something went wrong");
            }
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn test_stream_event_clone() {
        let event = StreamEvent::Start {
            name: "test".into(),
            input: serde_json::json!(null),
        };
        let cloned = event.clone();
        match cloned {
            StreamEvent::Start { name, .. } => assert_eq!(name, "test"),
            _ => panic!("expected Start"),
        }
    }

    #[test]
    fn test_into_stream_empty() {
        let items: Vec<Result<i32>> = vec![];
        let mut stream = into_stream(items);
        let result = futures::executor::block_on(stream.next());
        assert!(result.is_none());
    }

    #[test]
    fn test_into_stream_items() {
        let items: Vec<Result<i32>> = vec![Ok(1), Ok(2), Ok(3)];
        let stream = into_stream(items);
        let results: Vec<Result<i32>> = futures::executor::block_on(stream.collect());
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].as_ref().unwrap(), &1);
    }



    #[test]
    fn test_streaming_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ByteChunk>();
        assert_sync::<ByteChunk>();
        assert_send::<StreamEvent>();
        assert_sync::<StreamEvent>();
    }
}
